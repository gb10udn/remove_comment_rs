import win32com.client
import os
import glob
from typing import Any
import argparse


class VbaHandler:
    def __init__(self, src: str, *, is_visible=False):  # TODO: 240313 ThisWorkbook モジュールへの処理も追加せよ。
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

    
    def remove_existed_all_modules(self):
        for vb_component in self.workbook.VBProject.VBComponents:
            MODULE_TYPE = 1
            if vb_component.Type == MODULE_TYPE:
                self.workbook.VBProject.VBComponents.Remove(vb_component)
    

    def open_bas_file(self, path: str, *, remove_header=True) -> str | None:
        if os.path.exists(path):
            with open(path, 'r', encoding='utf-8') as f:
                vba_code = f.read()
            if remove_header == True:
                module_name = os.path.splitext(os.path.basename(path))[0]
                vba_code = vba_code.replace(f'Attribute VB_Name = "{module_name}"\n', '')
            return vba_code
        else:
            return None
    

    def add_module(self, vba_code: str, module_name: str, *, remove_default_code: bool=True):
        VBEXT_CT_STDMODULE = 1
        xlmodule = self.workbook.VBProject.VBComponents.Add(VBEXT_CT_STDMODULE)

        if remove_default_code == True:
            START_TO_DELETE_LINE_NUM = 1
            last_to_delete_line_num: int = xlmodule.CodeModule.CountOfLines
            xlmodule.CodeModule.DeleteLines(START_TO_DELETE_LINE_NUM, last_to_delete_line_num)  # INFO: 240312 to remove "Option Explicit" added by VBE
        
        xlmodule.Name = module_name
        xlmodule.CodeModule.AddFromString(vba_code)
    

    def save(self, dst: str):
        dst_abs = os.path.abspath(dst)  # INFO: 240310 win32api may force to use abs path
        assert self.abs_src != dst_abs,  f'OverWriteWarning: "src_excel_with_macro" must NOT be same with "dst". "src_excel_with_macro" / "dst" -> {self.abs_src}'
        assert os.path.exists(dst_abs) == False, f'OverWriteWarning: "dst" is already existed. dst -> {dst_abs}'

        XL_OPEN_XML_WORKBOOK_MACRO_ENABLED = 52
        self.workbook.SaveAs(dst_abs, FileFormat=XL_OPEN_XML_WORKBOOK_MACRO_ENABLED)


    def quit(self):
        self.workbook.Close()
        self.xl.Quit()


def update_vba_code(src_excel_with_macro: str, bas_src_dir: str, dst: str, *, is_visible: bool=False) -> None:
    """
    マクロファイルから VBA モジュールを全削除して、.bas ファイル から読みだした VBA モジュールを埋め込んで保存する。

    Parameters
    ----------
    src_excel_with_macro : str
        元となるマクロ付きエクセルブック。

    bas_src_dir : str
        .bas ファイルが格納されたディレクトリ。
        その直下の .bas ファイルを検索して、マクロ付きエクセルブックに新たに埋め込む。

    dst : str
        生成したマクロ付きエクセルブックの保存場所。

    is_visible : bool (default = False)
        エクセル操作を見せるか。デバッグ中のみ、True にする想定。

    Return
    ------
    None
    """
    vba_handler = VbaHandler(src=src_excel_with_macro, is_visible=is_visible)
    vba_handler.remove_existed_all_modules()

    bas_path_list = glob.glob(f'{bas_src_dir}/*.bas')
    for bas_path in bas_path_list:
        vba_code = vba_handler.open_bas_file(bas_path)
        if vba_code is not None:
            module_name = os.path.splitext(os.path.basename(bas_path))[0]
            vba_handler.add_module(vba_code=vba_code, module_name=module_name)
    
    vba_handler.save(dst=dst)
    vba_handler.quit()


if __name__ == '__main__':
    """
    # TODO: 240313 パスワードロックかけるといいかも？ビルド作業の補助になりそう。(https://qiita.com/feo52/items/150745ae0cc17cb5c866)
    Ex. python .\main.py --src "./misc/macro_sample_001.xlsm" --bas-dir "./dst_rmc/20240312_120734/macro_sample_001" --dst "dst.xlsm"
    """
    parser = argparse.ArgumentParser()
    parser.add_argument('--src', type=str, help='path of excel macro file')
    parser.add_argument('--bas-dir', type=str, help='base dir with .bas file')
    parser.add_argument('--dst', type=str, help='save path with macro with removed comment')

    args = parser.parse_args()

    assert args.src is not None, 'ArgError: args.src must not be None ...'
    assert args.bas_dir is not None, 'ArgError: args.bas_dir must not be None ...'
    assert args.dst is not None, 'ArgError: args.dst must not be None ...'

    update_vba_code(
        src_excel_with_macro=args.src,
        bas_src_dir=args.bas_dir,
        dst=args.dst,
    )