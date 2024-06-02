from os import get_terminal_size
from time import time

from colorama import Fore, Style, init
from getch import getch

from logger import Log
from text import EnglishFile

init(autoreset=False)


class Tester:
    def __init__(self, words: int = 200, offset: int = 0):
        self.text = EnglishFile.get_text(words)
        self.size = len(self.text)
        self.words = words
        self.prev = []
        self.index = 0
        self._offset = offset

    @property
    def offset(self):
        return self._offset or get_terminal_size().columns // 2 - 1

    def start(self):
        Log.warning("Make sure the command line is wide enough.")
        Log.info("Words count:", self.words)

        start_time = 0
        while self.index < self.size:
            print(f"\r{self.offset * 2 * ' '}\r" + self.underlay(), end="\r")
            print(self.overlay(), end="")
            self.check(getch())
            start_time = start_time or time()
        print("\r" + self.offset * 2 * " ", end="\r")
        Log.success("Your time:", time() - start_time, header="FINISH")

    def check(self, symbol: str):
        if symbol == "\x7f":
            if self.index != 0:
                self.prev.pop()
            self.index = max(0, self.index - 1)
            return
        self.prev.append(
            (Fore.LIGHTGREEN_EX if symbol == self.text[self.index] else Fore.LIGHTRED_EX) + symbol + Style.RESET_ALL)
        self.index += 1

    def underlay(self) -> str:
        if self.index < self.offset:
            return Fore.LIGHTBLACK_EX + self.text[:self.offset * 2] + Style.RESET_ALL
        return (Fore.LIGHTBLACK_EX +
                self.text[self.index - self.offset:self.index + self.offset] +
                Style.RESET_ALL)

    def overlay(self) -> str:
        if self.index < self.offset:
            return "".join(self.prev).strip()
        return "".join(self.prev[self.index - self.offset:]).strip()


if __name__ == "__main__":
    Tester(words=3).start()
