import rm

arg = '''Option Explicit
Sub PiyoPiyo()
    '
    ' docstring
    '
    MsgBox "PiyoPiyo"  ' TEST: fugafuga ???
    ' comment !!!
End Sub'''


def test_remove_unnecessary_comment():
    expected = '''Option Explicit
Sub PiyoPiyo()
    '
    ' docstring
    '
    MsgBox "PiyoPiyo"
    ' comment !!!
End Sub'''
    result = rm.remove_unnecessary_comment(arg, remove_comments=['TEST:'])
    assert result == expected


def test_remove_multiline_comment():
    expected = '''Option Explicit
Sub PiyoPiyo()
    MsgBox "PiyoPiyo"  ' TEST: fugafuga ???
    ' comment !!!
End Sub'''
    result = rm.remove_multiline_comment(arg)
    assert result == expected