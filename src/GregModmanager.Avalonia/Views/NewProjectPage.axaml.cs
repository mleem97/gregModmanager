using Avalonia.Controls;
using Avalonia.Interactivity;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Services;
using Microsoft.Extensions.DependencyInjection;

namespace GregModmanager.Avalonia.Views;

public partial class NewProjectPage : UserControl
{
    private readonly WorkspaceService _workspace;
    private readonly AppLogService _log;

    public NewProjectPage() => InitializeComponent();

    public NewProjectPage(WorkspaceService workspace, AppLogService log)
    {
        InitializeComponent();
        _workspace = workspace;
        _log = log;
    }

    private async void OnCreateTemplate(object? sender, RoutedEventArgs e)
    {
        try
        {
            var idx = TemplatePicker.SelectedIndex;
            if (idx < 0) idx = 0;
            var kind = (WorkshopTemplateKind)idx;
            var name = NameEntry.Text ?? string.Empty;
            var path = _workspace.CreateTemplateProject(name, kind);
            _log.Append($"Created template: {path}");
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("NewProject_Created"), path);
        }
        catch (Exception ex)
        {
            _log.Append($"Create template failed: {ex.Message}");
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("NewProject_CouldNotCreate"), ex.Message);
        }
    }
}
