import random
from pathlib import Path

CWD = Path.cwd() / "samples"


class FileManager:
    def __init__(self, filepath: str, filesize: int):
        self.filename = filepath
        self.filesize = filesize

    def get_text(self, words_count: int) -> str:
        words = ""
        for _ in range(words_count):
            words += " " + self.get_word()

        return words.strip()

    def get_word(self) -> str:
        word = ""

        with open(self.filename, mode="r", encoding="utf-8") as file:
            file.seek(random.randint(0, self.filesize - 50))
            while file.read(1) != "\n":
                pass
            while (c := file.read(1)) != "\n" and len(word) < 49:
                word += c

        return word.strip()


eng_path = CWD / "english.txt"
EnglishFile = FileManager(str(eng_path), eng_path.stat().st_size)
