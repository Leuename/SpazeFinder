fn main() {
    let mut attrs = tauri_build::Attributes::new();
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        let windows = tauri_build::WindowsAttributes::new().app_manifest(
            r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#,
        );
        attrs = attrs.windows_attributes(windows);
    }
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
