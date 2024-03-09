Set-Location ($PSScriptRoot)

Function test() {
    <#
    ŠÖ”ƒRƒƒ“ƒg‚Å‚·B
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}