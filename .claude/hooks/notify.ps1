# Notification / SubagentStop hook: a Windows toast so the user knows an agent finished or a
# decision is waiting. Reads the hook payload (JSON) on stdin; never blocks or fails the session.
param([string]$Kind = "Claude Code")
try {
    $raw = [Console]::In.ReadToEnd()
    $msg = ""
    if ($raw) {
        $p = $raw | ConvertFrom-Json
        if ($p.message) { $msg = [string]$p.message }
        elseif ($p.agent_type) { $msg = "Agent finished: " + [string]$p.agent_type }
        elseif ($p.hook_event_name -eq "SubagentStop") { $msg = "An agent finished." }
    }
    if (-not $msg) { $msg = "Claude Code needs your attention." }
    $msg = [Security.SecurityElement]::Escape($msg)
    $title = [Security.SecurityElement]::Escape("XBE Mod - " + $Kind)
    [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
    [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
    $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
    $xml.LoadXml("<toast><visual><binding template='ToastGeneric'><text>$title</text><text>$msg</text></binding></visual></toast>")
    $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
    # PowerShell's AppUserModelID, so the toast shows without registering an app
    $app = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe'
    [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($app).Show($toast)
} catch { }
exit 0
