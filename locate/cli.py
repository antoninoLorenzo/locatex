"""
Author: @antoninoLorenzo https://github.com/antoninoLorenzo
Version: 0.0
"""
import sys
import sqlite3
from argparse import Action, ArgumentParser
from pathlib import Path
from collections import namedtuple

INDEX_PATH = Path(Path('~').expanduser() / '.locatex' / 'index.db')
ItemFS = namedtuple(
    'ItemFS',
    ['abs_path', 'name', 'size', 'last_update', 'type']
)


class ValidateRegex(Action):
    """Validates user input regex"""

    def __call__(self, parser, namespace, values, option_string=None):
        # TODO: parse regex
        setattr(namespace, self.dest, values)


def update_index():
    """Launches Rust scanner"""


# noinspection SqlNoDataSourceInspection
def setup_index():
    """Creates database and updates file system index"""
    with sqlite3.connect(INDEX_PATH) as connection:
        cursor = connection.cursor()

        cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS fs (
                AbsPath TEXT PRIMARY KEY,
                Name TEXT NOT NULL,
                Size BIGINT NOT NULL,
                LastEdit DATE NOT NULL,
                Type TEXT CHECK ( Type in ('file', 'directory') )
            );
            """
        )

        cursor.execute("CREATE INDEX idx_name ON fs(Name)")

        connection.commit()

    update_index()


def setup_parser() -> ArgumentParser:
    """Configure ArgumentParser for locatex"""
    _parser = ArgumentParser()

    _parser.add_argument(
        'target',
        help='Name of file to search for.'
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


def search(target: str):
    """Search for a file or directory name in the filesystem"""
    with sqlite3.Connection(INDEX_PATH) as conn:
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM fs WHERE Name LIKE (?)", (f'%{target}%',))
        result = cursor.fetchall()
        if len(result) == 0:
            return None
        else:
            return [ItemFS(*r) for r in result]


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
        setup_index()
        updated = True

    if args.update == 1 and not updated:
        update_index()

    #  Run
    target_name = args.target
    output = search(target_name)
    if output is not None:
        for o in output:
            print(f'{o.type}: {o.abs_path}\t {o.size} bytes - {o.last_update}')
    else:
        print('No results found.')


if __name__ == '__main__':
    main()
