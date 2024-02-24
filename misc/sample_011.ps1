Set-Location ($PSScriptRoot)

Function test() {
    <#
    docstring
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}