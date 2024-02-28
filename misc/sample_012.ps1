Set-Location ($PSScriptRoot)

Function test() {
    <#
    Shift-jis のつもりで書いたコードです。
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}