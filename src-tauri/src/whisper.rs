use serde::Serialize;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Clone, Serialize)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading(f64),
    Ready,
    Error(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub variant: String,
    pub size_label: String,
}

pub struct WhisperEngine {
    inner: Mutex<WhisperInner>,
    model_dir: PathBuf,
    default_variant: String,
}

struct WhisperInner {
    status: ModelStatus,
    ctx: Option<WhisperContext>,
    current_variant: Option<String>,
}

fn model_url(variant: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        variant
    )
}

fn model_filename(variant: &str) -> String {
    format!("ggml-{}.bin", variant)
}

fn variant_size(variant: &str) -> &str {
    match variant {
        "tiny" => "75 MB",
        "base" => "142 MB",
        "small" => "466 MB",
        "medium" => "1.5 GB",
        "large-v3-turbo" => "1.5 GB",
        _ => "?",
    }
}

fn try_load_model(model_path: &std::path::Path, variant: &str) -> Option<WhisperContext> {
    if model_path.exists() {
        match WhisperContext::new_with_params(model_path, WhisperContextParameters::default()) {
            Ok(ctx) => {
                eprintln!("Whisper model '{}' loaded from {:?}", variant, model_path);
                Some(ctx)
            }
            Err(e) => {
                eprintln!("Failed to load Whisper model '{}': {}", variant, e);
                None
            }
        }
    } else {
        None
    }
}

impl WhisperEngine {
    pub fn new(app_data_dir: &std::path::Path, default_variant: &str) -> Self {
        let model_dir = app_data_dir.to_path_buf();
        let variants_to_check = [default_variant, "large-v3-turbo", "small", "tiny", "base"];
        let mut loaded = None;

        for v in &variants_to_check {
            let path = model_dir.join(model_filename(v));
            if let Some(ctx) = try_load_model(&path, v) {
                loaded = Some((v.to_string(), ctx));
                break;
            }
        }

        let inner = match loaded {
            Some((variant, ctx)) => WhisperInner {
                status: ModelStatus::Ready,
                ctx: Some(ctx),
                current_variant: Some(variant),
            },
            None => {
                let def_path = model_dir.join(model_filename(default_variant));
                eprintln!("No Whisper model found. Expected at {:?}", def_path);
                WhisperInner {
                    status: ModelStatus::NotDownloaded,
                    ctx: None,
                    current_variant: None,
                }
            }
        };

        WhisperEngine {
            inner: Mutex::new(inner),
            model_dir,
            default_variant: default_variant.to_string(),
        }
    }

    pub fn get_status(&self) -> ModelStatus {
        self.inner.lock().unwrap().status.clone()
    }

    pub fn get_model_info(&self) -> ModelInfo {
        let inner = self.inner.lock().unwrap();
        let variant = inner
            .current_variant
            .clone()
            .unwrap_or_else(|| self.default_variant.clone());
        let size_label = variant_size(&variant).to_string();
        ModelInfo { variant, size_label }
    }

    /// Starts model download in a background thread. Returns immediately.
    /// Progress can be tracked via `get_status()`.
    pub fn start_download(self: &Arc<Self>, variant: &str) -> Result<(), String> {
        {
            let inner = self.inner.lock().unwrap();
            if let ModelStatus::Downloading(_) = &inner.status {
                return Err("El modelo ya se está descargando".to_string());
            }
        }

        let variant = variant.to_string();
        let engine = Arc::clone(self);

        std::thread::spawn(move || {
            if let Err(e) = engine.download_blocking(&variant) {
                let mut inner = engine.inner.lock().unwrap();
                inner.status = ModelStatus::Error(format!("Error en descarga: {}", e));
            }
        });

        Ok(())
    }

    fn download_blocking(&self, variant: &str) -> Result<(), String> {
        let model_path = self.model_dir.join(model_filename(variant));
        let url = model_url(variant);

        {
            let mut inner = self.inner.lock().unwrap();
            inner.status = ModelStatus::Downloading(0.0);
            inner.current_variant = Some(variant.to_string());
        }

        fs::create_dir_all(&self.model_dir)
            .map_err(|e| format!("Error al crear directorio: {}", e))?;

        let response = reqwest::blocking::get(&url)
            .map_err(|e| format!("Error de conexión: {}", e))?;

        let total = response.content_length().unwrap_or(0);
        if total == 0 {
            return Err("No se pudo obtener el tamaño del modelo".to_string());
        }

        let mut file = fs::File::create(&model_path)
            .map_err(|e| format!("Error al crear archivo: {}", e))?;

        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192];
        let mut reader = BufReader::new(response);

        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|e| format!("Error de descarga: {}", e))?;
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])
                .map_err(|e| format!("Error al escribir archivo: {}", e))?;
            downloaded += n as u64;

            let pct = (downloaded as f64 / total as f64 * 100.0).min(99.9);
            let mut inner = self.inner.lock().unwrap();
            inner.status = ModelStatus::Downloading(pct);
        }

        let ctx = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
            .map_err(|e| format!("Error al cargar el modelo Whisper: {}", e))?;

        let mut inner = self.inner.lock().unwrap();
        inner.ctx = Some(ctx);
        inner.status = ModelStatus::Ready;
        Ok(())
    }

    pub fn transcribe(&self, audio_path: &str) -> Result<String, String> {
        let inner = self.inner.lock().unwrap();
        match &inner.status {
            ModelStatus::Ready => {}
            ModelStatus::Error(e) => return Err(format!("Modelo no disponible: {}", e)),
            ModelStatus::NotDownloaded => {
                return Err(
                    "Modelo no descargado. Ve a Configuración para descargarlo."
                        .to_string(),
                )
            }
            ModelStatus::Downloading(_) => {
                return Err(
                    "El modelo se está descargando. Espera a que termine.".to_string(),
                )
            }
        }
        let ctx = match inner.ctx.as_ref() {
            Some(ctx) => ctx,
            None => return Err("Modelo no cargado".to_string()),
        };

        let mut reader = hound::WavReader::open(audio_path)
            .map_err(|e| format!("Error al leer archivo WAV: {}", e))?;

        let spec = reader.spec();
        if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
            return Err("Formato WAV no soportado. Se requiere PCM 16-bit.".to_string());
        }

        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap_or(0)).collect();

        if samples.is_empty() {
            return Err("El archivo de audio está vacío".to_string());
        }

        let audio: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();

        let mut state = ctx
            .create_state()
            .map_err(|e| format!("Error al crear estado Whisper: {}", e))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("es"));
        params.set_print_progress(false);
        params.set_print_timestamps(false);

        state
            .full(params, &audio)
            .map_err(|e| format!("Error en transcripción: {}", e))?;

        let n_segments = state.full_n_segments();

        if n_segments == 0 {
            return Err(
                "No se detectó voz en el audio. Habla más claro o acércate al micrófono."
                    .to_string(),
            );
        }

        let mut text = String::new();
        for i in 0..n_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(s) = segment.to_str() {
                    let s = s.trim();
                    if !s.is_empty() {
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        text.push_str(s);
                    }
                }
            }
        }

        let text = text.trim().to_string();
        if text.is_empty() {
            return Err(
                "No se detectó voz en el audio. Habla más claro o acércate al micrófono."
                    .to_string(),
            );
        }

        Ok(text)
    }
}
