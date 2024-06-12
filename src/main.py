from argparse import ArgumentParser
from os import get_terminal_size
from time import time, sleep

from colorama import Fore, Style, init
from getch import getch

import logger as Log
from text import EnglishFile

init(autoreset=False)


class Tester:
    def __init__(self, words: int = 200, offset: int = 0, timer: int = 60):
        self.text = EnglishFile.get_text(words)
        self.size = len(self.text)
        self.words = words
        self.timer = timer
        self.correct = 0
        self.prev = []
        self.index = 0
        self._offset = offset

    @property
    def offset(self):
        return self._offset or get_terminal_size().columns // 2 - 1

    def start(self):
        Log.warning("Make sure the command line is wide enough.")
        sleep(1.5)
        Log.info("Words count:", self.words)
        Log.info("Timer: {:.2f}s".format(self.timer) if self.timer else "Unlimited time.")
        sleep(1)
        start_time = 0
        while self.index < self.size:
            print(f"\r{self.offset * 2 * ' '}\r" + self.underlay(), end="\r")
            print(self.overlay(), end="")
            self.check(getch())
            start_time = start_time or time()
            if self.timer and time() - start_time >= self.timer: break

        print("\r" + self.offset * 2 * " ", end="\r")
        delta_time = min(time() - start_time, self.timer)
        accurate = self.correct * 100 / self.index

        Log.info("Test ended!")
        sleep(1)
        Log.warning("{:.1f} %".format(accurate), header="ACCURATE")
        sleep(.5)
        Log.success(int((self.correct + 1) * (60 / delta_time)), header="CPM", end="\t")
        Log.success(int(((self.correct + 1) / 5) * (60 / delta_time)), header="WPM")

    def check(self, symbol: str):
        if symbol == "\x7f":
            if self.index != 0:
                self.correct -= (self.prev.pop() == (Fore.LIGHTGREEN_EX +
                                                     self.text[self.index - 1] +
                                                     Style.RESET_ALL))
            self.index = max(0, self.index - 1)
            return
        if symbol == self.text[self.index]:
            self.prev.append(Fore.LIGHTGREEN_EX + symbol + Style.RESET_ALL)
            self.correct += 1
        else:
            self.prev.append(Fore.LIGHTRED_EX + symbol + Style.RESET_ALL)
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
    parser = ArgumentParser(prog="Speed-Typing", description="Test your speed of typing just in your terminal!")
    parser.add_argument("-w", "--words", metavar="N", default=200, type=int, help="Count of words (Default - 200).")
    parser.add_argument("-o", "--offset", metavar="N", type=int,
                        help="Maximum offset from the cursor to the edges of the terminal (Default - half the length of the terminal).")
    parser.add_argument("-t", "--time", metavar="seconds", dest="timer", default=60, type=int,
                        help="Time to pass the test (In seconds, default - 60s).")

    Tester(**parser.parse_args().__dict__).start()
