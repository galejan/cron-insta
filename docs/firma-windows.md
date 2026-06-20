# Guía de firma de código para Windows

Firmar el binario de Cronista con un certificado Code Signing reduce las advertencias de Windows Defender y SmartScreen al instalar la aplicación.

---

## Etapa 1 — Obtener el certificado

### Autoridades certificantes (CA) recomendadas

| CA | Tipo | Coste anual aprox. | SmartScreen inmediato |
|----|------|--------------------|------------------------|
| [SSL.com](https://ssl.com) | Standard / EV | 200–300 € | No (reputación necesaria) |
| [DigiCert](https://digicert.com) | Standard / EV | 300–600 € | Sí (solo EV) |
| [Sectigo](https://sectigo.com) | Standard / EV | 200–350 € | No (reputación necesaria) |

### Standard vs EV (Extended Validation)

- **Standard**: validación de dominio u organización. Las primeras semanas Windows SmartScreen muestra advertencia hasta que el certificado acumula «reputación» (descargas suficientes sin reportes de malware).
- **EV**: validación presencial de la empresa + token USB físico. SmartScreen confía de inmediato. Más caro y requiere empresa constituida.

### Para individuos (sin empresa)

SSL.com y Sectigo permiten certificados Code Signing para personas físicas. El proceso de validación incluye verificación de identidad (documento oficial, selfie, etc.).

---

## Etapa 2 — Obtener el archivo `.pfx`

Una vez aprobada la solicitud, la CA entrega:

- Un archivo **`.pfx`** o **`.p12`** (clave privada + certificado)
- Una **contraseña** para protegerlo

**Guardá el `.pfx` en un lugar seguro.** Es la llave de tu aplicación. Si lo perdés o se filtra, no podés revocar builds ya distribuidos sin que Windows los rechace.

---

## Etapa 3 — Configurar la firma

### Opción A: GitHub Actions (recomendado)

El workflow `build.yml` que ya existe en el repositorio lee las variables `TAURI_PRIVATE_KEY` y `TAURI_KEY_PASSWORD`. Solo hay que agregarlas como secretos.

**Paso 1 — Convertir el `.pfx` a base64**

```bash
# En tu máquina local (Linux/macOS)
base64 -w0 tu-certificado.pfx > pfx-base64.txt
```

En Windows (PowerShell):
```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("C:\ruta\tu-certificado.pfx")) | Out-File pfx-base64.txt
```

**Paso 2 — Agregar los secretos en GitHub**

1. Ir a `https://github.com/galejan/cronista/settings/secrets/actions`
2. Agregar dos **Repository secrets**:

| Nombre | Valor |
|--------|-------|
| `TAURI_PRIVATE_KEY` | El contenido de `pfx-base64.txt` (una sola línea larga) |
| `TAURI_KEY_PASSWORD` | La contraseña del `.pfx` |

**Paso 3 — Pushear a `main`**

El workflow se ejecuta automáticamente y Tauri firma los binarios sin cambios adicionales.

---

### Opción B: Firma local en Windows

Agregar la configuración de firma en `src-tauri/tauri.conf.json`:

```json
"bundle": {
  "active": true,
  "targets": "all",
  "windows": {
    "signCommand": "signtool sign /fd SHA256 /f C:\\ruta\\certificado.pfx /p TU_PASSWORD /tr http://timestamp.digicert.com /td SHA256 %1"
  }
}
```

Luego compilar normalmente:

```bash
pnpm tauri build --bundles msi,nsis
```

> `signtool.exe` viene con el Windows SDK. Si no está en el PATH, usar la ruta completa (ej. `C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe`).

---

## Verificación

Después de firmar, verificar que el binario está correctamente sellado:

```bash
signtool verify /pa /v src-tauri/target/release/cronista.exe
```

Debe mostrar `Successfully verified` y los detalles del certificado.

---

## Renovación

Los certificados Code Signing vencen cada 1 o 2 años. Renovar **antes** del vencimiento para no interrumpir la cadena de confianza. Al renovar, actualizar los secretos de GitHub Actions inmediatamente.
