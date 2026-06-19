use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading(f64),
    Ready,
    Error(String),
}

impl ModelStatus {
    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        matches!(self, ModelStatus::Ready)
    }
}

pub struct WhisperEngine;

impl WhisperEngine {
    pub fn new() -> Self {
        WhisperEngine
    }

    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        false
    }

    pub fn get_status(&self) -> ModelStatus {
        ModelStatus::NotDownloaded
    }

    #[allow(dead_code)]
    pub fn load_model(&self) -> Result<(), String> {
        Err("No hay modelo descargado. Usa download_model primero.".to_string())
    }

    pub fn download_model(&self) -> Result<(), String> {
        Err("Whisper requiere descargar un modelo de ~1.5 GB con conexión a internet.\nPara grabar recordatorios por voz se usa el reconocedor integrado de Windows (SAPI), que funciona sin descargas adicionales.".to_string())
    }

    pub fn transcribe(&self, audio_path: &str) -> Result<String, String> {
        let script = format!(
            r#"
Add-Type -AssemblyName System.Speech
$recognizer = New-Object System.Speech.Recognition.SpeechRecognitionEngine
try {{
    $recognizer.SetInputToWaveFile('{0}')
    $result = $recognizer.Recognize()
    if ($result -ne $null) {{
        Write-Output $result.Text
    }} else {{
        Write-Output ''
    }}
}} catch {{
    Write-Error $_.Exception.Message
}} finally {{
    $recognizer.Dispose()
}}
"#,
            audio_path.replace('\'', "''")
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("SetInputToWaveFile") {
                return Err(
                    "Error de audio: formato WAV no soportado. Usa PCM mono 16kHz."
                        .to_string(),
                );
            }
            if stderr.contains("SpeechRecognitionEngine") {
                return Err(
                    "Error: No se pudo inicializar el reconocedor de voz de Windows."
                        .to_string(),
                );
            }
            return Err(format!("Error en transcripción: {}", stderr));
        }

        let text = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        if text.is_empty() {
            return Err(
                "No se detectó voz en el audio. Habla más claro o acércate al micrófono."
                    .to_string(),
            );
        }

        Ok(text)
    }
}
