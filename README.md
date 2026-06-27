# Recordatorio

**Asistente personal de recordatorios por voz** — Crea recordatorios hablando, sin escribir. Usa inteligencia artificial local para transcribir tu voz y entender fechas y horas en lenguaje natural.

Captura tu voz, la convierte a texto, interpreta automáticamente cuándo quieres que te recuerde algo, y te notifica en el momento exacto. Todo corre **100% local** y **offline**, sin conexión a internet ni servicios en la nube.

---

## ✨ Funcionalidades

### 🎤 Creación por voz
Habla y el sistema transcribe automáticamente usando **Whisper** (IA local). Entiende frases como:
- *"Recuérdame comprar leche mañana a las 9"*
- *"Reunión en 30 minutos"*
- *"Llamar al dentista el lunes a las 3pm"*
- *"Ejercicio cada día a las 7am"*

### 🧠 Interpretación inteligente de fechas
El analizador integrado reconoce expresiones en español como:
- Relativas: *"en 15 minutos", "dentro de 2 horas", "en 3 días"*
- Específicas: *"mañana", "hoy", "el viernes"*
- Con hora: *"a las 10:30", "a las 5 de la tarde"*
- Repetición: *"cada hora", "cada 30 minutos", "cada día"*

### 🔔 Notificaciones con sonido
- Notificaciones nativas del sistema operativo
- Sonido personalizable (archivo `.wav`)
- Notificación anticipada (configurable: 1, 2, 5, 10, 15... minutos antes)
- Recordatorios **repetitivos** (cada 15 min, 30 min, 1 hora, 2 horas, 1 día, etc.)
- Reprogramación automática de recordatorios repetitivos

### ⌨️ Atajos globales de teclado
| Atajo | Acción |
|-------|--------|
| `Ctrl+Alt+R` | Iniciar grabación de voz |
| `Ctrl+Alt+O` | Mostrar/Ocultar ventana |
| `Ctrl+Alt+N` | Nuevo recordatorio |
| `Ctrl+Alt+S` | Posponer último (5 min) |
| `Ctrl+Alt+D` | Completar último |

Todos los atajos son **personalizables** desde la configuración.

### 🖥️ Bandeja del sistema
- Se minimiza a la bandeja al cerrar
- Menú contextual rápido con acceso a funciones principales
- Sigue funcionando en segundo plano

### 💾 Base de datos dual
- **Modo local** — Los recordatorios se guardan en SQLite en tu PC
- **Modo compartido** — Los recordatorios se sincronizan en **SQL Server**, permitiendo compartirlos entre varios equipos en la misma red
- Cambia entre modos en cualquier momento desde la configuración

### ⚙️ Configuración completa
- Inicio automático con Windows
- Tema claro/oscuro
- Sonido de notificación personalizable
- Intervalo de revisión de recordatorios
- Tiempo de aviso anticipado
- Prueba de notificación
- Descarga de modelos Whisper (tiny, base, small, medium, large)
- Atajos de teclado editables
- Actualizaciones automáticas desde GitHub

### 📋 Gestión completa
- **Dashboard** con recordatorios pendientes, vencidos y notificados
- **Historial** con filtro por estado (completados, cancelados, notificados)
- Editar, completar, cancelar, posponer o eliminar recordatorios
- Solo una instancia a la vez (no se abren ventanas duplicadas)

---

## 🛠️ Tecnologías usadas

### Frontend
- **React 19** + **TypeScript** — UI moderna y tipada
- **Vite 7** — Build rápido y desarrollo eficiente
- **Tailwind CSS 3** — Estilos utilitarios y tema oscuro
- **Zustand** — Estado global ligero

### Backend
- **Rust** con **Tauri v2** — App nativa de escritorio, segura y rápida
- **Tiberius** — Conexión nativa a SQL Server
- **Rusqlite** — Base de datos SQLite embebida
- **Whisper.rs** — Transcripción de voz local con Whisper.cpp (IA offline)
- **Chrono** — Manejo de fechas y horas

### Instalador
- **NSIS** — Instalador nativo para Windows (generado con el bundler de Tauri)
- Actualizaciones automáticas vía GitHub Releases

---

## 🚀 ¿Por qué usar Recordatorio?

| Problema | Solución |
|----------|----------|
| Escribir recordatorios toma tiempo | **Habla y listo** — transcribe automáticamente |
| Las apps de notas no notifican | **Notificaciones nativas** con sonido y anticipación |
| Depender de la nube o internet | **100% local y offline** — tu privacidad primero |
| Recordatorios en un solo dispositivo | **Modo compartido** con SQL Server en tu red |
| Sin atajos rápidos | **5 atajos globales** para acciones frecuentes |
| Se cierra la app y se pierden | **Bandeja del sistema** + inicio automático con Windows |

---

## 📦 Instalación

Descarga el instalador desde la [sección de Releases](https://github.com/JDFO1999/Recordatorio/releases):

```
Recordatorio_x64-setup.exe
```

Ejecútalo y sigue los pasos. La app se abrirá automáticamente al finalizar.

> Nota: La primera vez que uses la grabación por voz, la app te guiará para descargar el modelo Whisper (selecciona "tiny" o "base" para empezar — son rápidos de descargar y ligeros).

---

## 🔧 Desarrollo

### Requisitos
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Comandos

```bash
# Instalar dependencias
npm install

# Desarrollo
npm run tauri dev

# Build para producción (genera instalador NSIS y MSI)
npm run tauri build
```

El instalador se genera en `src-tauri/target/release/bundle/nsis/`.

---

## 📄 Licencia

Este proyecto es de uso interno. Desarrollado con ❤️ por JodixSystem.
