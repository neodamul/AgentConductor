use agentconductor::ui::picker::{FolderPurpose, folder_picker_title};

#[test]
fn folder_picker_titles_match_the_requested_workspace_action() {
    assert_eq!(
        folder_picker_title(FolderPurpose::Project, "en"),
        "Choose a project folder"
    );
    assert_eq!(
        folder_picker_title(FolderPurpose::WorkspaceSettingsExport, "en"),
        "Choose where to export workspace settings"
    );
    assert_eq!(
        folder_picker_title(FolderPurpose::WorkspaceSettingsImport, "en"),
        "Choose a folder containing workspace settings"
    );
    assert_eq!(
        folder_picker_title(FolderPurpose::WorkspaceSettingsExport, "ko"),
        "워크스페이스 설정을 내보낼 폴더 선택"
    );
    assert_eq!(
        folder_picker_title(FolderPurpose::WorkspaceSettingsImport, "ko"),
        "워크스페이스 설정이 있는 폴더 선택"
    );
}
