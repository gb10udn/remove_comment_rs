Set-Location ($PSScriptRoot)

Function test() {
    <#
    関数コメントです。
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}