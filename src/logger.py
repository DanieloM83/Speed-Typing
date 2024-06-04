import sys

from colorama import Fore, Style, Back, init

init(autoreset=True)


def notset(*args, header="NOTSET", **kwargs):
    print(f"{Fore.WHITE}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def debug(*args, header="DEBUG", **kwargs):
    print(f"{Fore.CYAN}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def info(*args, header="INFO", **kwargs):
    print(f"{Fore.BLUE}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def success(*args, header="SUCCESS", **kwargs):
    print(f"{Fore.GREEN}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def warning(*args, header="WARNING", **kwargs):
    print(f"{Fore.YELLOW}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def error(*args, header="ERROR", **kwargs):
    print(f"{Fore.RED}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def critical(*args, header="CRITICAL", **kwargs):
    print(f"{Fore.MAGENTA}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def alert(*args, header="ALERT", **kwargs):
    print(f"{Fore.LIGHTRED_EX}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()


def emergency(*args, header="EMERGENCY", **kwargs):
    print(f"{Fore.LIGHTWHITE_EX + Back.RED}[{header}]{Style.RESET_ALL}", *args, **kwargs)
    sys.stdout.flush()
