import win32com.client
import os
import glob
from typing import Any
import argparse


def insert_vba_code(src_excel_with_macro: str, bas_src_dir: str, dst: str, *, is_visible: bool=False) -> None:
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
    assert src_excel_with_macro != dst, f'src_excel_with_macro must NOT be same with dst. src_excel_with_macro / dst -> {src_excel_with_macro}'  # INFO: 240312 上書き防止
    assert os.path.exists(dst) == False, f'dst is already existed. dst -> {dst}'  # INFO: 240312 上書き防止

    xl: Any = win32com.client.Dispatch('Excel.Application')
    xl.Visible = is_visible
    workbook = xl.Workbooks.Open(os.path.abspath(src_excel_with_macro))  # INFO: 240310 絶対パスに直すこと。win32api のベースディレクトリで判定している気がする。

    # [START] remove existed bas modules
    for vb_component in workbook.VBProject.VBComponents:
        MODULE_TYPE = 1
        if vb_component.Type == MODULE_TYPE:
            workbook.VBProject.VBComponents.Remove(vb_component)
    # [END] remove existed bas modules

    # [START] search .bas files and embed
    bas_path_list = glob.glob(f'{bas_src_dir}/*.bas')
    for bas_path in bas_path_list:
        with open(bas_path, 'r', encoding='utf-8') as f:
            vba_code = f.read()

        VBEXT_CT_STDMODULE = 1
        xlmodule = workbook.VBProject.VBComponents.Add(VBEXT_CT_STDMODULE)
        module_name = os.path.splitext(os.path.basename(bas_path))[0]
        xlmodule.Name = module_name
        xlmodule.CodeModule.AddFromString(vba_code)
    # [END] search .bas files and embed

    # [START] save and quit
    dst = os.path.abspath(dst)
    XL_OPEN_XML_WORKBOOK_MACRO_ENABLED = 52
    workbook.SaveAs(dst, FileFormat=XL_OPEN_XML_WORKBOOK_MACRO_ENABLED)
    workbook.Close()
    xl.Quit()
    # [END] save and quit


if __name__ == '__main__':
    """
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

    insert_vba_code(
        src_excel_with_macro=args.src,
        bas_src_dir=args.bas_dir,
        dst=args.dst,
    )