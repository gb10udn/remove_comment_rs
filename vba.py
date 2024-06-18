import rm

import win32com.client
import traceback

import os
import json
from typing import Any
import argparse
import logging
import datetime


class VbaHandler:
    def __init__(self, src: str, *, is_visible: bool=False):
        """
        マクロ付きエクセルブックの VBA を扱うためのクラス。
        特に、VBA モジュールの書き込み、削除を処理する。(Rust で処理実行できなかったため、Python の win32api を利用することにした。)
        """
        ext = os.path.splitext(src)[-1]
        assert ext == '.xlsm', f'ArgError: extension of "src" must be ".xlsm", not {ext}'

        self.xl: Any = win32com.client.Dispatch('Excel.Application')
        self.xl.Visible = is_visible
        self.abs_src = os.path.abspath(src)  # INFO: 240310 win32api may force to use abs path
        self.workbook = self.xl.Workbooks.Open(self.abs_src)
    

    def update_vba_code_with_removed_unnecessary_comments(self, *, remove_comments: list, remove_multiline_comment: bool, remove_test_code: bool) -> None:
        for component in self.workbook.VBProject.VBComponents:
            MODULE_TYPE = 1
            EXCEL_OBJECT_TYPE = 100
            if component.Type in [MODULE_TYPE, EXCEL_OBJECT_TYPE]:
                START_LINE_IDX = 1
                last_line_idx = component.CodeModule.CountOfLines
                if last_line_idx > START_LINE_IDX:
                    # [START] obtain new_code
                    code = component.CodeModule.Lines(START_LINE_IDX, last_line_idx)
                    code = rm.remove_unnecessary_comment(code, remove_comments=remove_comments)

                    if remove_multiline_comment == True:
                        code = rm.remove_multiline_comment(code)

                    if remove_test_code == True:
                        code = rm.remove_test_code(code)
                    # [END] obtain new_code


                    # [START] update new code
                    component.CodeModule.DeleteLines(START_LINE_IDX, last_line_idx)
                    component.CodeModule.AddFromString(code)
                    # [END] update new code
    

    def save(self, dst: str) -> None:
        dst_abs = os.path.abspath(dst)  # INFO: 240310 win32api may force to use abs path
        assert self.abs_src != dst_abs,  f'OverWriteWarning: "src" must NOT be same with "dst". "src" / "dst" -> {self.abs_src}'
        assert os.path.exists(dst_abs) == False, f'OverWriteWarning: "dst" is already existed. dst -> {dst_abs}'

        XL_OPEN_XML_WORKBOOK_MACRO_ENABLED = 52
        self.workbook.SaveAs(dst_abs, FileFormat=XL_OPEN_XML_WORKBOOK_MACRO_ENABLED)


    def quit(self) -> None:
        self.workbook.Close()
        self.xl.Quit()


####
        

def update_vba_code_with_removed_unnecessary_comments(src: str, dst: str, *, remove_comments: list[str], remove_multiline_comment: bool=True, remove_test_code: bool=True, is_visible: bool=False) -> None:
    """
    マクロファイルから VBA モジュールを全削除して、.bas ファイル から読みだした VBA モジュールを埋め込んで保存する。

    Parameters
    ----------
    src : str
        元となるマクロ付きエクセルブック。

    dst : str
        生成したマクロ付きエクセルブックの保存場所。
    
    remove_comments : list
        コメント削除の対象とする文字列。

    remove_multiline_comment : bool (default : True)
        複数行の連続コメント (docstring がこれにあたる想定) を削除するかどうか。

    remove_test_code : bool (default = True)
        Function or Sub において、TEST_ から始まるものを test_code と定義し、それを削除するかどうか。

    is_visible : bool (default = False)
        エクセル操作を見せるか。デバッグ中のみ、True にする想定。

    Return
    ------
    None
    """
    vba_handler = VbaHandler(src=src, is_visible=is_visible)
    vba_handler.update_vba_code_with_removed_unnecessary_comments(remove_comments=remove_comments, remove_multiline_comment=remove_multiline_comment, remove_test_code=remove_test_code)
    vba_handler.save(dst=dst)
    vba_handler.quit()


def set_logging(dst_dir: str='./log_py') -> None:
    stream_handler = logging.StreamHandler()
    stream_handler.setLevel(logging.WARNING)

    now = datetime.datetime.now().strftime('%Y-%m-%d_%H-%M-%S')
    log_path = f'{dst_dir}/{now}.log'
    os.makedirs(os.path.dirname(log_path), exist_ok=True)

    file_handler = logging.FileHandler(log_path, encoding='utf-8')
    file_handler.setLevel(logging.DEBUG)

    logging.basicConfig(
        format='%(asctime)s\t%(levelname)s\t%(name)s\t%(message)s',
        datefmt='%Y-%m-%d %H:%M:%S',
        handlers=[stream_handler, file_handler],
        level=logging.DEBUG,
    )


if __name__ == '__main__':
    """
    Ex. python ./vba.py --src "./misc/macro_sample_001.xlsm" --dst "./misc/macro_sample_001_editted.xlsm"  --remove-multiline-comment 1 --remove-excel-macro-test-code 1
    """
    set_logging()
    logger = logging.getLogger(__name__)

    parser = argparse.ArgumentParser()
    parser.add_argument('--src', type=str, help='path of excel macro file')
    parser.add_argument('--dst', type=str, help='save path with macro with removed comment')
    parser.add_argument('--remove-multiline-comment', type=int)
    parser.add_argument('--remove-excel-macro-test-code', type=int)

    args = parser.parse_args()
    logger.debug(args)

    try:
        assert args.src is not None, 'ArgError: args.src must not be None ...'
        assert args.dst is not None, 'ArgError: args.dst must not be None ...'
        assert args.remove_multiline_comment is not None, 'ArgError: args.remove_multiline_comment must not be None ...'
        assert args.remove_excel_macro_test_code is not None, 'ArgError: args.remove_excel_macro_test_code must not be None ...'

        assert args.src != args.dst, f'DuplicateError: --src and --dst must NOT be same ... -> "{args.src}"'
        assert args.remove_multiline_comment in [0, 1], f'ArgError: --remove-multiline-comment must be 0 or 1, not {args.remove_multiline_comment}'
        assert args.remove_excel_macro_test_code in [0, 1], f'ArgError: --remove-excel-macro-test-code must be 0 or 1, not {args.remove_multiline_comment}'
    
    except AssertionError as err:
        logger.debug(err)
        raise Exception(err)

    try:
        CONFIG_PATH = './config.json'
        CONFIG_KEY = 'remove_comments'
        with open(CONFIG_PATH) as f:
            config = json.load(f)
        remove_comments = config[CONFIG_KEY]
    
    except FileNotFoundError:
        message = f'FileNotFoundError: CONFIG_PATH -> {CONFIG_PATH}'
        logger.warning(message)
        raise Exception(message)
    except KeyError:
        message = f'KeyError: CONFIG_KEY -> {CONFIG_KEY}'
        logger.warning(message)
        raise Exception(message)
    except Exception as err:
        detail_message = ''.join(traceback.TracebackException.from_exception(err).format())
        logger.warning(detail_message)
        raise Exception(detail_message)

    try:
        update_vba_code_with_removed_unnecessary_comments(
            src=args.src,
            dst=args.dst,
            remove_comments=remove_comments,
            remove_multiline_comment=bool(args.remove_multiline_comment),
            remove_test_code=bool(args.remove_excel_macro_test_code),
        )
    except Exception as err:
        detail_message = ''.join(traceback.TracebackException.from_exception(err).format())
        logger.warning(detail_message)