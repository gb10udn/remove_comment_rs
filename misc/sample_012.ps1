Set-Location ($PSScriptRoot)

Function test() {
    <#
    Shift-jis �̂���ŏ������R�[�h�ł��B
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}