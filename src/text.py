import random
from pathlib import Path

CWD = Path.cwd() / "samples"


class FileManager:
    def __init__(self, filepath: Path, filesize: int = 0):
        self.filename = filepath
        self.filesize = filesize if filesize else filepath.stat().st_size

    def get_text(self, words_count: int) -> str:
        words = ""
        for _ in range(words_count):
            words += " " + self.get_word()

        return words.strip()

    def get_word(self) -> str:
        word = ""

        with open(self.filename, mode="r", encoding="utf-8") as file:
            file.seek(random.randint(0, self.filesize - 5))
            while file.read(1) != "\n":
                pass
            while (c := file.read(1)) != "\n" and file.tell() != self.filesize:
                word += c

        return word.strip()
