using System;
using Microsoft.Maui.Controls;
using Microsoft.Maui.Graphics;
using Microsoft.Maui.Dispatching;
using Microsoft.Maui.ApplicationModel;
using GregModmanager.Localization;
using GregModmanager.Services;
using GregModmanager.Services.Auth;
using GregModmanager.Services.Install;
using GregModmanager.Models.Auth;

namespace GregModmanager;

public partial class AppShell : Shell
{
        private readonly SteamWorkshopService _steam;
        private readonly ISessionManager _session;
        private readonly IInstallIntentClient _installIntent;
        private bool _steamStatusTimerStarted;

        public AppShell(
            ProjectsPage projects,
            NewProjectPage newProject,
            MyUploadsPage uploads,
            ModManagerPage modManager,
            SettingsPage settings,
            SteamWorkshopService steam,
            ISessionManager session,
            IInstallIntentClient installIntent)
        {
                _steam = steam;
                _session = session;
                _installIntent = installIntent;
                _session.StateChanged += UpdateAuthConnectionUi;
                _session.ProtocolInvoked += uri =>
                {
                    MainThread.BeginInvokeOnMainThread(async () =>
                    {
                        if (uri.Contains("/auth/callback"))
                        {
                            await _session.HandleProtocolCallbackAsync(uri);
                        }
                        else if (uri.Contains("/install/intent"))
                        {
                            await _installIntent.HandleIntentAsync(uri);
                        }
                    });
                };

                DebugSessionLog.Write("H2", "AppShell.ctor", "before_init", null);
                InitializeComponent();
                DebugSessionLog.Write("H2", "AppShell.ctor", "after_init", null);

                Loaded += OnShellLoaded;

                Items.Add(new FlyoutItem
                {
                        Title = S.Get("Tab_Projects"),
                        Items = { new ShellContent { Content = projects, Route = nameof(ProjectsPage) } },
                });
                Items.Add(new FlyoutItem
                {
                        Title = S.Get("Tab_New"),
                        Items = { new ShellContent { Content = newProject, Route = nameof(NewProjectPage) } },
                });
                Items.Add(new FlyoutItem
                {
                        Title = S.Get("Tab_MyUploads"),
                        Items = { new ShellContent { Content = uploads, Route = nameof(MyUploadsPage) } },
                });

                if (SettingsPage.IsModStoreEnabled())
                {
                        Items.Add(new FlyoutItem
                        {
                                Title = S.Get("Tab_ModStore"),
                                Items = { new ShellContent { Content = modManager, Route = nameof(ModManagerPage) } },
                        });
                }

                Items.Add(new FlyoutItem
                {
                        Title = S.Get("Tab_Settings"),
                        Items = { new ShellContent { Content = settings, Route = nameof(SettingsPage) } },
                });

                Routing.RegisterRoute(nameof(EditorPage), typeof(EditorPage));
                Routing.RegisterRoute(nameof(NativeConfigEditorPage), typeof(NativeConfigEditorPage));
                Routing.RegisterRoute(nameof(ItemDetailPage), typeof(ItemDetailPage));
        }

        private async void OnAuthTileTapped(object sender, EventArgs e)
        {
            if (_session.State == SessionState.Authenticated)
            {
                await _session.LogoutAsync();
            }
            else
            {
                await _session.StartBrowserLoginAsync();
            }
        }

        private async void OnShellLoaded(object? sender, EventArgs e)
        {
                await _session.InitializeAsync();

                var args = Environment.GetCommandLineArgs();
                foreach (var arg in args)
                {
                    if (arg.StartsWith("greg://", StringComparison.OrdinalIgnoreCase))
                    {
                        if (arg.Contains("/auth/callback"))
                        {
                            await _session.HandleProtocolCallbackAsync(arg);
                            break;
                        }
                        else if (arg.Contains("/install/intent"))
                        {
                            await _installIntent.HandleIntentAsync(arg);
                            break;
                        }
                    }
                }

                if (_steamStatusTimerStarted)
                {
                        return;
                }

                _steamStatusTimerStarted = true;
                Loaded -= OnShellLoaded;
                UpdateSteamConnectionUi();
                UpdateGregApiConnectionUi();
                var dispatcher = Application.Current?.Dispatcher;
                if (dispatcher is null) return;

                dispatcher.StartTimer(TimeSpan.FromSeconds(2), () =>
                {
                        MainThread.BeginInvokeOnMainThread(() =>
                        {
                            UpdateSteamConnectionUi();
                            UpdateGregApiConnectionUi();
                        });
                        return true;
                });
        }

        private void UpdateAuthConnectionUi()
        {
            MainThread.BeginInvokeOnMainThread(() =>
            {
                switch (_session.State)
                {
                    case SessionState.Authenticated:
                        AuthStatusLed.Fill = new SolidColorBrush(Color.FromArgb("#61F4D8"));
                        AuthLogoTile.BackgroundColor = Color.FromArgb("#0D3835");
                        AuthStatusText.TextColor = Color.FromArgb("#C0FCF6");
                        AuthStatusText.Text = $"Logged in as {_session.CurrentSession?.User?.DisplayName ?? "User"}";
                        break;
                    case SessionState.LoginPending:
                    case SessionState.Refreshing:
                        AuthStatusLed.Fill = new SolidColorBrush(Color.FromArgb("#FFC107"));
                        AuthLogoTile.BackgroundColor = Color.FromArgb("#1A2A1012");
                        AuthStatusText.TextColor = Color.FromArgb("#FFC107");
                        AuthStatusText.Text = "Logging in...";
                        break;
                    default:
                        AuthStatusLed.Fill = new SolidColorBrush(Color.FromArgb("#D7383B"));
                        AuthLogoTile.BackgroundColor = Color.FromArgb("#1A2A1012");
                        AuthStatusText.TextColor = Color.FromArgb("#7FBFB8");
                        AuthStatusText.Text = "Login To Datacentermods.com";
                        break;
                }
            });
        }

        private void UpdateSteamConnectionUi()
        {
                if (_steam.TryGetSteamReady(out var userName))
                {
                        SteamStatusLed.Fill = new SolidColorBrush(Color.FromArgb("#61F4D8"));
                        SteamLogoTile.BackgroundColor = Color.FromArgb("#0D3835");
                        SteamStatusText.TextColor = Color.FromArgb("#C0FCF6");
                        SteamStatusText.Text = "Steam Connected";
                }
                else
                {
                        SteamStatusLed.Fill = new SolidColorBrush(Color.FromArgb("#D7383B"));
                        SteamLogoTile.BackgroundColor = Color.FromArgb("#1A2A1012");
                        SteamStatusText.TextColor = Color.FromArgb("#7FBFB8");
                        SteamStatusText.Text = "Steam Disconnected";
                }
        }

        private void UpdateGregApiConnectionUi()
        {
                if (_session.State == SessionState.Authenticated && _session.CurrentSession != null)
                {
                        GregApiStatusLed.Fill = new SolidColorBrush(Color.FromArgb("#61F4D8"));
                        GregApiLogoTile.BackgroundColor = Color.FromArgb("#0D3835");
                        GregApiStatusText.TextColor = Color.FromArgb("#C0FCF6");
                        GregApiStatusText.Text = "gregAPI Online";
                }
                else
                {
                        GregApiStatusLed.Fill = new SolidColorBrush(Color.FromArgb("#D7383B"));
                        GregApiLogoTile.BackgroundColor = Color.FromArgb("#1A2A1012");
                        GregApiStatusText.TextColor = Color.FromArgb("#7FBFB8");
                        GregApiStatusText.Text = "gregAPI Offline";
                }
        }
}
