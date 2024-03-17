"""
Author: @antoninoLorenzo https://github.com/antoninoLorenzo
Version: 0.0
"""
import sys
from argparse import Action, ArgumentParser
from pathlib import Path

INDEX_PATH = Path(Path('~').expanduser() / '.locatex' / 'index.db')


class ValidateRegex(Action):

    def __call__(self, parser, namespace, values, option_string=None):
        # TODO: parse regex
        setattr(namespace, self.dest, values)


def update_index():
    """Launches Rust scanner"""
    pass


def setup_index():
    """Creates database and updates file system index"""
    # ...
    update_index()


def setup_parser() -> ArgumentParser:
    """Configure ArgumentParser for locatex"""
    _parser = ArgumentParser()

    _parser.add_argument(
        'target',
        help='Name of file to search for.'
    )

    _parser.add_argument(
        '--regex',
        default=None,
        help='Finds target with regular expression.'
    )

    _parser.add_argument(
        '--update',
        choices=(0, 1),
        default=0,
        help='Updates the index.'
    )

    _parser.add_argument(
        '--insensitive',
        choices=(0, 1),
        default=0,
        help='Finds target with case insensitive option.'
    )

    return _parser


def main():
    """Main function called by locatex.py"""
    parser = setup_parser()
    args = parser.parse_args()

    if args.target is None:
        print('Target is required.')
        sys.exit(1)

    #  Update db
    updated = False
    if not INDEX_PATH.exists():
        if not INDEX_PATH.parent.exists():
            INDEX_PATH.parent.mkdir()
        setup_parser()
        updated = True

    if args.update == 1 and not updated:
        update_index()

    #  Run


if __name__ == '__main__':
    main()
