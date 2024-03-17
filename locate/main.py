import os
from python.utils import performance_test, display_top


@performance_test
def index_files(target):
    """
    should return a dict with:
    for files {name: {path}}
    for dirs {name: {path, list of items (files or dirs)}}
    """
    target_fs = {}
    for root, dirs, files in os.walk(target):
        for name in dirs:
            pass
        for name in files:
            pass
    return target_fs


if __name__ == "__main__":
    res, time, snapshot = index_files('C://')
    print(f'Done in: {time:.2f}')
