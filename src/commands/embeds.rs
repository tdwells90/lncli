use std::fs;
use std::path::Path;

use crate::cli::{EmbedsArgs, EmbedsCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::FileUploadResponse;
use crate::utils::error::CliError;
use crate::utils::output;

const MAX_FILE_SIZE: u64 = 20 * 1024 * 1024; // 20 MB

pub async fn execute(client: &GraphqlClient, args: EmbedsArgs) -> Result<(), CliError> {
    match args.command {
        EmbedsCommand::Upload { file } => upload(client, &file).await,
        EmbedsCommand::Download {
            url,
            output: output_path,
            overwrite,
        } => download(client, &url, output_path, overwrite).await,
    }
}

async fn upload(client: &GraphqlClient, file_path: &str) -> Result<(), CliError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(CliError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {file_path}"),
        )));
    }

    let metadata = fs::metadata(path)?;
    let size = metadata.len();

    if size > MAX_FILE_SIZE {
        return Err(CliError::FileTooLarge {
            path: file_path.to_string(),
            size,
            max: MAX_FILE_SIZE,
        });
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let content_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    // Step 1: Get presigned upload URL
    let response: FileUploadResponse = client
        .request(
            queries::FILE_UPLOAD_URL,
            serde_json::json!({
                "contentType": content_type,
                "filename": filename,
                "size": size as i64,
            }),
        )
        .await?;

    let upload_file = response.file_upload.upload_file;

    // Step 2: Upload the file to the presigned URL
    let file_bytes = fs::read(path)?;
    let mut request = client
        .http_client()
        .put(&upload_file.upload_url)
        .header("Content-Type", &content_type)
        .body(file_bytes);

    // Add any extra headers from the upload response
    if let Some(headers) = &upload_file.headers {
        for h in headers {
            request = request.header(&h.key, &h.value);
        }
    }

    let upload_response = request.send().await?;

    if !upload_response.status().is_success() {
        let status = upload_response.status();
        let body = upload_response.text().await.unwrap_or_default();
        return Err(CliError::Other(format!(
            "Upload failed with HTTP {status}: {body}"
        )));
    }

    output::print_json(&serde_json::json!({
        "success": true,
        "assetUrl": upload_file.asset_url,
        "filename": filename,
        "size": size,
        "contentType": content_type,
    }));

    Ok(())
}

async fn download(
    client: &GraphqlClient,
    url: &str,
    output_path: Option<String>,
    overwrite: bool,
) -> Result<(), CliError> {
    let dest = match output_path {
        Some(ref p) => p.clone(),
        None => {
            // Extract filename from URL
            url.rsplit('/')
                .next()
                .and_then(|s| s.split('?').next())
                .unwrap_or("download")
                .to_string()
        }
    };

    let dest_path = Path::new(&dest);
    if dest_path.exists() && !overwrite {
        return Err(CliError::Other(format!(
            "File already exists: {dest}. Use --overwrite to replace."
        )));
    }

    // Determine auth method based on URL
    let request = if url.contains("uploads.linear.app") {
        // Use Bearer token for Linear's storage
        client
            .http_client()
            .get(url)
            .header("Authorization", client.token())
    } else {
        // Signed URL - no auth needed
        client.http_client().get(url)
    };

    let response = request.send().await?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(CliError::Other(format!(
            "Download failed with HTTP {status}"
        )));
    }

    let bytes = response.bytes().await?;
    fs::write(dest_path, &bytes)?;

    output::print_json(&serde_json::json!({
        "success": true,
        "path": dest,
        "size": bytes.len(),
    }));

    Ok(())
}
