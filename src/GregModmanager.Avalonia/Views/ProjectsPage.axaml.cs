using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.VisualTree;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Services;
using Microsoft.Extensions.DependencyInjection;

namespace GregModmanager.Avalonia.Views;

public partial class ProjectsPage : UserControl
{
    private readonly WorkspaceService _workspace = null!;
    private readonly AppLogService _log = null!;
    private readonly List<ProjectItemVm> _allProjects = new();
    private string _searchQuery = "";

    public ProjectsPage() => InitializeComponent();

    public ProjectsPage(WorkspaceService workspace, AppLogService log)
    {
        InitializeComponent();
        _workspace = workspace;
        _log = log;
        _workspace.EnsureWorkspaceStructure();

        var migrated = _workspace.MigrateLegacyProjects();
        if (migrated > 0) _log.Append(S.Format("Projects_Migrated", migrated));

        WorkspacePathLabel.Text = _workspace.WorkspaceRoot;
        _log.LineAppended += OnLogAppended;
        ReloadProjects();
        _log.Append(S.Get("Projects_Ready"));
    }

    private void OnLogAppended(object? sender, EventArgs e)
    {
        LogText.Text = string.Join(Environment.NewLine, _log.Lines);
    }

    private void ReloadProjects()
    {
        _allProjects.Clear();
        foreach (var p in _workspace.ScanProjects())
        {
            _allProjects.Add(new ProjectItemVm(p, _workspace));
        }
        ApplySearchFilter();
    }

    private void OnSearchTextChanged(object? sender, TextChangedEventArgs e)
    {
        _searchQuery = SearchBox.Text ?? "";
        ApplySearchFilter();
    }

    private void ApplySearchFilter()
    {
        var q = _searchQuery.Trim();
        var filtered = _allProjects.Where(vm =>
            string.IsNullOrEmpty(q) ||
            vm.Name.Contains(q, StringComparison.OrdinalIgnoreCase) ||
            vm.RootPath.Contains(q, StringComparison.OrdinalIgnoreCase) ||
            vm.Title.Contains(q, StringComparison.OrdinalIgnoreCase) ||
            (!string.IsNullOrEmpty(vm.Tags) && vm.Tags.Contains(q, StringComparison.OrdinalIgnoreCase))
        ).ToList();
        ProjectList.ItemsSource = filtered;
    }

    private void OnRefreshProjects(object? sender, RoutedEventArgs e)
    {
        ReloadProjects();
        _log.Append(S.Get("Projects_Refreshed"));
    }

    private void OnProjectTapped(object? sender, PointerPressedEventArgs e)
    {
        if (sender is Border border && border.DataContext is ProjectItemVm vm)
        {
            var editor = App.Services.GetRequiredService<EditorPage>();
            editor.LoadProject(vm.RootPath);
            if (this.GetVisualRoot() is MainWindow mw) mw.NavigateTo(editor);
        }
    }
}

public sealed class ProjectItemVm
{
    public ProjectItemVm(WorkshopProject project, WorkspaceService workspace)
    {
        Name = project.Name;
        RootPath = project.RootPath;
        var meta = workspace.LoadMetadata(project.RootPath);
        Title = string.IsNullOrWhiteSpace(meta.Title) ? project.Name : meta.Title;
        Tags = meta.Tags.Count > 0 ? string.Join(", ", meta.Tags) : "";
        HasTags = meta.Tags.Count > 0;
        IsPublished = meta.PublishedFileId != 0;
        PublishedStatus = IsPublished ? "Published" : "";

        var checks = UploadDependencyChecker.Check(project.RootPath, meta);
        var errors = checks.Count(c => c.Severity == UploadCheckSeverity.Error);
        var warnings = checks.Count(c => c.Severity == UploadCheckSeverity.Warning);

        if (errors > 0)
            ReadinessText = $"{errors} error(s)";
        else if (warnings > 0)
            ReadinessText = $"Ready ({warnings} warning(s))";
        else
            ReadinessText = "Ready";

        if (errors > 0)
            ReadinessColor = "#D7383B";
        else if (warnings > 0)
            ReadinessColor = "#D7A23B";
        else
            ReadinessColor = "#61F4D8";
    }

    public string Name { get; }
    public string RootPath { get; }
    public string Title { get; }
    public string Tags { get; }
    public bool HasTags { get; }
    public bool IsPublished { get; }
    public string PublishedStatus { get; }
    public string ReadinessText { get; }
    public string ReadinessColor { get; }
}
