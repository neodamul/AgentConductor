use super::workspace;

#[derive(Clone, Copy)]
pub(super) enum Pending {
    Save,
    BackgroundRefresh,
    BackgroundTaskDetails,
    BackgroundCoordinator,
    Refresh,
    Action,
    ToolDecision { resume_after: bool },
    Briefing,
    Coordinator,
    TaskDetails,
    Start,
    Evidence,
    ScopeFailure,
    ScopeRecoveryPreview,
    ScopeRecoveryApply,
    Prepare,
    NextPhase,
    SelectGoal,
    PrepareRefresh,
    Metadata,
    MetadataRefresh,
    PickFolder,
    PickExecutable,
    ProfileSave,
    CatalogSave,
    CatalogRefresh,
    ProfileRefresh,
    ProfileRetryRefresh,
    ResolveExecutable,
    Edit,
    StartRefresh,
    OpenRun,
    CachePreview,
    CacheClear,
    Navigation,
    NavigationRefresh,
    StopPlanning,
    StopPlanningRefresh,
    ShareFolder(workspace::sharing::Action),
    ShareExport,
    SharePreview,
    ShareImport,
    McpDiscover,
    McpPreview,
    McpApply,
    McpApplyRefresh,
    McpStatus,
    McpSave,
    McpRefresh,
    MessengerLoad,
    MessengerTest,
    MessengerSave,
    MessengerRefresh,
    SignalSettingsLoad,
    SignalSettingsSave,
    UpdateSourceLoad,
    UpdateSourceSave,
    ChatStart,
    ChatRefresh,
    ChatReset,
    OptimizeStart,
    OptimizeRefresh,
    Update,
}

impl Pending {
    pub(super) fn handled_by_settings(self) -> bool {
        matches!(
            self,
            Self::PickFolder
                | Self::ShareFolder(_)
                | Self::ShareExport
                | Self::SharePreview
                | Self::ShareImport
                | Self::PickExecutable
                | Self::ResolveExecutable
                | Self::ProfileSave
                | Self::CatalogSave
                | Self::CatalogRefresh
                | Self::ProfileRefresh
                | Self::ProfileRetryRefresh
                | Self::McpSave
                | Self::McpRefresh
                | Self::MessengerLoad
                | Self::MessengerTest
                | Self::MessengerSave
                | Self::MessengerRefresh
                | Self::SignalSettingsLoad
                | Self::SignalSettingsSave
                | Self::UpdateSourceLoad
                | Self::UpdateSourceSave
                | Self::Update
                | Self::Save
                | Self::Prepare
                | Self::NextPhase
                | Self::SelectGoal
                | Self::Metadata
                | Self::MetadataRefresh
                | Self::ChatStart
                | Self::ChatRefresh
                | Self::ChatReset
                | Self::OptimizeStart
                | Self::OptimizeRefresh
        )
    }
}
