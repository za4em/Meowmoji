from pathlib import Path

import keras
import numpy as np
import tensorflow as tf


MODEL_DIR = Path(__file__).resolve().parent
LABELS_PATH = MODEL_DIR / "labels.txt"
TRAIN_PATH = MODEL_DIR / "train.tsv"
HOLDOUT_PATH = MODEL_DIR / "holdout.tsv"
VOCAB_PATH = MODEL_DIR / "vocab.txt"
TFLITE_PATH = MODEL_DIR / "meowmoji.tflite"

TOKEN_SLOTS = 4
EMBEDDING_WIDTH = 32
LABEL_COUNT = 100
PAD_ID = 0
PAD_TOKEN = "<pad>"
EPOCHS = 20
BATCH_SIZE = 16
LEARNING_RATE = 0.001
RANDOM_SEED = 1


def read_label_list(path: Path) -> list[str]:
    """Return the emoji in file order. The line number is the score index."""
    labels = []
    for line in path.read_text(encoding="utf-8").splitlines():
        emoji = line.strip()
        if emoji:
            labels.append(emoji)
    if len(labels) != LABEL_COUNT:
        raise ValueError(f"{path} has {len(labels)} labels, expected {LABEL_COUNT}")
    return labels


def label_indexes_by_emoji(labels: list[str]) -> dict[str, int]:
    """Map each emoji character to its line number."""
    return {emoji: index for index, emoji in enumerate(labels)}


def read_phrase_rows(path: Path) -> list[tuple[str, str]]:
    """Return (phrase, emoji) pairs from a tab-separated file."""
    rows = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        parts = line.split("\t")
        if len(parts) != 2:
            raise ValueError(f"{path}:{line_number} needs a phrase, a tab, and an emoji")
        phrase, emoji = parts
        rows.append((phrase, emoji))
    return rows


def phrases_from_rows(rows: list[tuple[str, str]]) -> list[str]:
    """Keep only the phrase column."""
    return [phrase for phrase, _emoji in rows]


def words_in_phrase(phrase: str) -> list[str]:
    """Split a phrase into lowercase words."""
    return phrase.lower().split()


def vocabulary_from_phrases(phrases: list[str]) -> list[str]:
    """Return the distinct training words, sorted. Pad is not included."""
    words = {word for phrase in phrases for word in words_in_phrase(phrase)}
    return sorted(words)


def word_ids_from_vocabulary(vocabulary: list[str]) -> dict[str, int]:
    """Give each word an id starting at 1. Id 0 is reserved for pad."""
    return {word: index + 1 for index, word in enumerate(vocabulary)}


def require_word_count(phrase: str, words: list[str]) -> None:
    """Reject an empty phrase or one that does not fit the four slots."""
    if not words:
        raise ValueError(f"phrase has no words: {phrase!r}")
    if len(words) > TOKEN_SLOTS:
        raise ValueError(f"phrase has more than {TOKEN_SLOTS} words: {phrase!r}")


def token_id_for_known_word(word: str, word_ids: dict[str, int]) -> int:
    """Return the id for a word that was seen in training."""
    try:
        return word_ids[word]
    except KeyError:
        raise ValueError(f"word {word!r} is not in the training vocabulary") from None


def pad_token_ids(token_ids: list[int]) -> list[int]:
    """Extend a short row with pad ids until it has four slots."""
    missing = TOKEN_SLOTS - len(token_ids)
    return token_ids + [PAD_ID] * missing


def token_ids_for_phrase(phrase: str, word_ids: dict[str, int]) -> list[int]:
    """Turn one phrase into four token ids."""
    words = words_in_phrase(phrase)
    require_word_count(phrase, words)
    ids = [token_id_for_known_word(word, word_ids) for word in words]
    return pad_token_ids(ids)


def label_index_for_emoji(emoji: str, label_indexes: dict[str, int]) -> int:
    """Return the score index for an emoji that is in the label list."""
    try:
        return label_indexes[emoji]
    except KeyError:
        raise ValueError(f"emoji {emoji!r} is not in labels.txt") from None


def encode_phrase_rows(
    rows: list[tuple[str, str]],
    word_ids: dict[str, int],
    label_indexes: dict[str, int],
) -> tuple[np.ndarray, np.ndarray]:
    """Turn phrase rows into a token-id array and a label-index array."""
    token_rows = []
    labels = []
    for phrase, emoji in rows:
        token_rows.append(token_ids_for_phrase(phrase, word_ids))
        labels.append(label_index_for_emoji(emoji, label_indexes))
    return np.array(token_rows, dtype=np.int32), np.array(labels, dtype=np.int32)


def average_real_token_vectors(token_ids, vectors):
    """Average the word vectors and leave the pad slots out."""
    present = keras.ops.cast(keras.ops.not_equal(token_ids, PAD_ID), dtype=vectors.dtype)
    present_columns = keras.ops.expand_dims(present, axis=-1)
    summed = keras.ops.sum(vectors * present_columns, axis=1)
    count = keras.ops.sum(present, axis=1, keepdims=True)
    return summed / count


def build_emoji_model(vocabulary_size: int, label_count: int) -> keras.Model:
    """Build token ids, word vectors, a masked average, and 100 scores."""
    token_ids = keras.Input(shape=(TOKEN_SLOTS,), dtype="int32", name="token_ids")
    vectors = keras.layers.Embedding(
        input_dim=vocabulary_size,
        output_dim=EMBEDDING_WIDTH,
        name="word_vectors",
    )(token_ids)
    averaged = average_real_token_vectors(token_ids, vectors)
    scores = keras.layers.Dense(
        label_count,
        use_bias=False,
        activation="softmax",
        name="emoji_scores",
    )(averaged)
    return keras.Model(inputs=token_ids, outputs=scores, name="meowmoji")


def compile_for_training(model: keras.Model) -> None:
    """Choose how the scores are compared with the correct emoji index."""
    model.compile(
        optimizer=keras.optimizers.Adam(learning_rate=LEARNING_RATE),
        loss="sparse_categorical_crossentropy",
        metrics=["accuracy"],
    )


def fit_training_rows(model: keras.Model, token_ids: np.ndarray, label_indexes: np.ndarray) -> None:
    """Update the word vectors and the score matrix from the training rows."""
    model.fit(
        token_ids,
        label_indexes,
        epochs=EPOCHS,
        batch_size=BATCH_SIZE,
        shuffle=True,
        verbose=1,
    )


def fraction_correct(model: keras.Model, token_ids: np.ndarray, label_indexes: np.ndarray) -> float:
    """Return the share of rows whose winning score is the correct emoji."""
    _loss, accuracy = model.evaluate(token_ids, label_indexes, verbose=0)
    return float(accuracy)


def write_vocabulary(path: Path, vocabulary: list[str]) -> None:
    """Write one word per line. Line 0 is pad. The line number is the token id."""
    lines = [PAD_TOKEN, *vocabulary]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def export_tflite(model: keras.Model, path: Path) -> None:
    """Write one .tflite file that scores a single row of four token ids."""
    token_ids = keras.Input(shape=(TOKEN_SLOTS,), batch_size=1, dtype="int32", name="token_ids")
    fixed = keras.Model(token_ids, model(token_ids, training=False))
    converter = tf.lite.TFLiteConverter.from_keras_model(fixed)
    path.write_bytes(converter.convert())


def fix_random_seed(seed: int) -> None:
    """Make the starting numbers and the row order repeatable."""
    keras.utils.set_random_seed(seed)


def main() -> None:
    fix_random_seed(RANDOM_SEED)
    labels = read_label_list(LABELS_PATH)
    label_indexes = label_indexes_by_emoji(labels)
    training_rows = read_phrase_rows(TRAIN_PATH)
    holdout_rows = read_phrase_rows(HOLDOUT_PATH)
    vocabulary = vocabulary_from_phrases(phrases_from_rows(training_rows))
    word_ids = word_ids_from_vocabulary(vocabulary)
    train_tokens, train_labels = encode_phrase_rows(training_rows, word_ids, label_indexes)
    holdout_tokens, holdout_labels = encode_phrase_rows(holdout_rows, word_ids, label_indexes)
    model = build_emoji_model(vocabulary_size=len(vocabulary) + 1, label_count=len(labels))
    compile_for_training(model)
    fit_training_rows(model, train_tokens, train_labels)
    holdout_accuracy = fraction_correct(model, holdout_tokens, holdout_labels)
    write_vocabulary(VOCAB_PATH, vocabulary)
    export_tflite(model, TFLITE_PATH)
    print(f"holdout accuracy {holdout_accuracy:.3f}")
    print(f"wrote {VOCAB_PATH.name}")
    print(f"wrote {TFLITE_PATH.name}")


if __name__ == "__main__":
    main()
