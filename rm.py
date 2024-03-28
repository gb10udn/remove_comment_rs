import re
from dataclasses import dataclass


def remove_unnecessary_comment(vba_code: str, *, remove_comments: list) -> str:
    pattern_str = '|'.join([r" *' *" + mark + '.*(?:\r?\n|$)' for mark in remove_comments])
    pattern_re = re.compile(pattern_str)
    return pattern_re.sub('\n', vba_code)


def remove_test_code(vba_code: str) -> str:
    """
    TEST_ から始まる、Sub or Function を削除する。
    (エクセルマクロで、TEST_ から始まるものは単体テストであるとして開発されていることが前提。)
    """
    pattern = re.compile(r'((Public|Private) (Sub|Function) TEST_.+?End (Sub|Function))|((Sub|Function) TEST_.+?End (Sub|Function))', re.DOTALL)
    return re.sub(pattern, '', vba_code)


@dataclass
class CodeInfo:
    starts_with_single_quatation: bool


def remove_multiline_comment(vba_code: str) -> str:
    splited_arg = vba_code.split('\n')
    code_info_list = []
    for idx, one_line in enumerate(splited_arg):
        one_line = one_line.replace(' ', '')
        if len(one_line) > 0:
            code_info_list.append(CodeInfo(starts_with_single_quatation=one_line[0] == "'"))
        else:
            code_info_list.append(CodeInfo(starts_with_single_quatation=False))

    START_IDX = 0
    end_idx = len(code_info_list) - 1
    remove_idx_list = []
    for idx in range(len(code_info_list)):
        current: CodeInfo = code_info_list[idx]
        if idx == START_IDX:
            next: CodeInfo = code_info_list[idx + 1]
            if (current.starts_with_single_quatation == True) and (next.starts_with_single_quatation == True):
                remove_idx_list.append(idx)
        elif idx == end_idx:
            previous: CodeInfo = code_info_list[idx - 1]
            if (current.starts_with_single_quatation == True) and (previous.starts_with_single_quatation == True):
                remove_idx_list.append(idx)
        else:
            next: CodeInfo = code_info_list[idx + 1]  # type: ignore
            previous: CodeInfo = code_info_list[idx - 1]  # type: ignore
            if (current.starts_with_single_quatation == True) and ((previous.starts_with_single_quatation == True) or (next.starts_with_single_quatation == True)):
                remove_idx_list.append(idx)

    return '\n'.join([one_line for idx, one_line in enumerate(splited_arg) if idx not in remove_idx_list])