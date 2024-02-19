import os

def hoge(arg: str) -> str | None:  # HACK: 240213 mypy を導入する。
    """
    docstring だよ！  # TODO: 240213 docstring 中の # TODO: はどうなるのかな？
    """
    if os.path.exists(arg):
        return arg  # FIXME: 240213 piyopiyo
    else:
        return None  # EDIT: 240213 hogehoge