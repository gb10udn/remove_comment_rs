Set-Location ($PSScriptRoot)

Function test() {
    <#
    �֐��R�����g�ł��B
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}