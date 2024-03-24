import win32com.client
import os
import json
from typing import Any
import argparse
import rm


class VbaHandler:
    def __init__(self, src: str, *, is_visible=False):  # TODO: 240324 TEST と付いたプロシージャも削除するとよいかも？
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
    

    def update_vba_code_with_removed_unnecessary_comments(self, *, remove_comments: list, remove_multiline_comment: bool):
        for component in self.workbook.VBProject.VBComponents:
            MODULE_TYPE = 1
            EXCEL_OBJECT_TYPE = 100
            if component.Type in [MODULE_TYPE, EXCEL_OBJECT_TYPE]:
                # [START] obtain new_code
                START_LINE_IDX = 1
                last_line_idx = component.CodeModule.CountOfLines
                code = component.CodeModule.Lines(START_LINE_IDX, last_line_idx)
                code = rm.remove_unnecessary_comment(code, remove_comments=remove_comments)

                if remove_multiline_comment == True:
                    code = rm.remove_multiline_comment(code)
                # [END] obtain new_code


                # [START] update new code
                component.CodeModule.DeleteLines(START_LINE_IDX, last_line_idx)
                component.CodeModule.AddFromString(code)
                # [END] update new code
    

    def save(self, dst: str):
        dst_abs = os.path.abspath(dst)  # INFO: 240310 win32api may force to use abs path
        assert self.abs_src != dst_abs,  f'OverWriteWarning: "src" must NOT be same with "dst". "src" / "dst" -> {self.abs_src}'
        assert os.path.exists(dst_abs) == False, f'OverWriteWarning: "dst" is already existed. dst -> {dst_abs}'

        XL_OPEN_XML_WORKBOOK_MACRO_ENABLED = 52
        self.workbook.SaveAs(dst_abs, FileFormat=XL_OPEN_XML_WORKBOOK_MACRO_ENABLED)


    def quit(self):
        self.workbook.Close()
        self.xl.Quit()


####
        

def update_vba_code_with_removed_unnecessary_comments(src: str, dst: str, *, remove_comments: list[str], remove_multiline_comment: bool=True, is_visible: bool=False) -> None:
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

    is_visible : bool (default = False)
        エクセル操作を見せるか。デバッグ中のみ、True にする想定。

    Return
    ------
    None
    """
    vba_handler = VbaHandler(src=src, is_visible=is_visible)
    vba_handler.update_vba_code_with_removed_unnecessary_comments(remove_comments=remove_comments, remove_multiline_comment=remove_multiline_comment)
    vba_handler.save(dst=dst)
    vba_handler.quit()


if __name__ == '__main__':  # TODO: 240313 パスワードロックかけるといいかも？ビルド作業の補助になりそう。(https://qiita.com/feo52/items/150745ae0cc17cb5c866)
    """
    Ex. python ./vba.py --src "./misc/macro_sample_001.xlsm" --dst "./misc/macro_sample_001_editted.xlsm"
    """
    parser = argparse.ArgumentParser()
    parser.add_argument('--src', type=str, help='path of excel macro file')
    parser.add_argument('--dst', type=str, help='save path with macro with removed comment')
    parser.add_argument('--remove-multiline-comment', type=int)

    args = parser.parse_args()

    assert args.src is not None, 'ArgError: args.src must not be None ...'
    assert args.dst is not None, 'ArgError: args.dst must not be None ...'
    assert args.remove_multiline_comment is not None, 'ArgError: args.remove_multiline_comment must not be None ...'

    assert args.src != args.dst, f'DuplicateError: --src and --dst must NOT be same ... -> "{args.src}"'
    assert args.remove_multiline_comment in [0, 1], f'ArgError: --remove-multiline-comment must be 0 or 1, not {args.remove_multiline_comment}'

    try:
        CONFIG_PATH = './config.json'
        CONFIG_KEY = 'remove_comments'
        with open(CONFIG_PATH) as f:
            config = json.load(f)
        remove_comments = config[CONFIG_KEY]
    
    except FileNotFoundError:
        raise Exception(f'FileNotFoundError: CONFIG_PATH -> {CONFIG_PATH}')
    except KeyError:
        raise Exception(f'KeyError: CONFIG_KEY -> {CONFIG_KEY}')
    except:
        raise Exception('InternalError: unknown type of error ...')

    update_vba_code_with_removed_unnecessary_comments(
        src=args.src,
        dst=args.dst,
        remove_comments=remove_comments,
        remove_multiline_comment=bool(args.remove_multiline_comment),
    )