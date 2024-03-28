import rm


def test_remove_unnecessary_comment():
    vba_code = '''Option Explicit
Sub PiyoPiyo()
    '
    ' docstring
    '
    MsgBox "PiyoPiyo"  ' TEST: fugafuga ???
    ' comment !!!
End Sub'''

    expected = '''Option Explicit
Sub PiyoPiyo()
    '
    ' docstring
    '
    MsgBox "PiyoPiyo"
    ' comment !!!
End Sub'''
    result = rm.remove_unnecessary_comment(vba_code, remove_comments=['TEST:'])
    assert result == expected


def test_remove_multiline_comment():
    vba_code = '''Option Explicit
Sub PiyoPiyo()
    '
    ' docstring
    '
    MsgBox "PiyoPiyo"  ' TEST: fugafuga ???
    ' comment !!!
End Sub'''

    expected = '''Option Explicit
Sub PiyoPiyo()
    MsgBox "PiyoPiyo"  ' TEST: fugafuga ???
    ' comment !!!
End Sub'''
    result = rm.remove_multiline_comment(vba_code)
    assert result == expected


def test_remove_test_code():
    vba_code = '''Option Explicit
Sub PiyoPiyo()
    '
    ' docstring
    '
    MsgBox "PiyoPiyo"  ' TEST: fugafuga ???
    ' comment !!!
End Sub

Sub TEST_PiyoPiyo()
    MsgBox "this is excel macro test sub"
End Sub'''

    expected = '''Option Explicit
Sub PiyoPiyo()
    '
    ' docstring
    '
    MsgBox "PiyoPiyo"  ' TEST: fugafuga ???
    ' comment !!!
End Sub

'''

    result = rm.remove_test_code(vba_code)
    assert result == expected