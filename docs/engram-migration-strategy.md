# Estrategia de migración de sesiones Engram

## Rutas clave

| Recurso | Ruta |
|---------|------|
| DB local actual | `~/.engram/engram.db` (+ `.db-shm`, `.db-wal`) |
| DB usuario antiguo | `/home/alex/usuario_antiguo/alex/.engram/engram.db` |
| Cloud server | `http://vpsengram.duckdns.org:18080` |
| Token cloud | `ENGRAM_CLOUD_TOKEN` (en entorno) |

## Workflow general: importar DB antigua completa

```bash
# 1. Backup de la DB actual
cp ~/.engram/engram.db /tmp/backup-current.db
cp ~/.engram/engram.db-shm /tmp/backup-current.db-shm
cp ~/.engram/engram.db-wal /tmp/backup-current.db-wal

# 2. Reemplazar temporalmente con DB antigua
rm -f ~/.engram/engram.db ~/.engram/engram.db-shm ~/.engram/engram.db-wal
cp /home/alex/usuario_antiguo/alex/.engram/engram.db ~/.engram/engram.db
cp /home/alex/usuario_antiguo/alex/.engram/engram.db-shm ~/.engram/engram.db-shm
cp /home/alex/usuario_antiguo/alex/.engram/engram.db-wal ~/.engram/engram.db-wal

# 3. Exportar a JSON
engram export /tmp/engram-old-export.json

# 4. Restaurar DB actual
rm -f ~/.engram/engram.db ~/.engram/engram.db-shm ~/.engram/engram.db-wal
cp /tmp/backup-current.db ~/.engram/engram.db
cp /tmp/backup-current.db-shm ~/.engram/engram.db-shm
cp /tmp/backup-current.db-wal ~/.engram/engram.db-wal

# 5. Importar el JSON en la DB actual
engram import /tmp/engram-old-export.json
```

## Workflow: merge de proyecto con rename (ej. cronista → cron-insta)

Cuando el proyecto cambió de nombre y hay que unificar:

```bash
# 1. Exportar DB completa
engram export /tmp/engram-full.json

# 2. Filtrar y renombrar con Python
python3 << 'EOF'
import json
with open('/tmp/engram-full.json') as f:
    data = json.load(f)

old_name = 'cronista'
new_name = 'cron-insta'

for section in ['sessions', 'observations', 'prompts']:
    filtered = [item for item in data[section] if item.get('project') == old_name]
    for item in filtered:
        item['project'] = new_name
    data[section] = filtered

with open('/tmp/engram-merged.json', 'w') as f:
    json.dump(data, f, indent=2, default=str)
EOF

# 3. Importar (crea nuevas entradas con nuevos IDs, no sobrescribe)
engram import /tmp/engram-merged.json
```

## Workflow: mover sesiones entre proyectos (SQL)

Las sesiones no se pueden mover con herramientas de Engram. Si `engram import`
no las mueve (porque ya existen con el mismo ID), usar SQL:

```python
import sqlite3
conn = sqlite3.connect('/home/alex/.engram/engram.db')
conn.execute("UPDATE sessions SET project = 'nuevo-proyecto' WHERE project = 'viejo-proyecto'")
conn.commit()
conn.close()
```

**Precaución**: `engram delete project <name> --hard` falla con FK constraint si
las sesiones son referenciadas por observaciones de otro proyecto. Solución:
primero soft-delete (`engram delete project <name>` sin `--hard`) y luego el
UPDATE SQL para mover las sesiones huérfanas.

## Workflow: enrolar y sincronizar a la nube

```bash
# 1. Enrolar proyecto (requisito previo)
engram cloud enroll <nombre-proyecto>

# 2. Sincronizar
engram sync --cloud --project <nombre-proyecto>

# 3. Verificar estado
engram sync --status --cloud --project <nombre-proyecto>
```

## Lecciones aprendidas

- **No existe flag `--data-dir` ni `ENGRAM_HOME`**: la DB siempre está en `~/.engram/`. Para exportar de otra DB hay que copiarla temporalmente.
- **`engram import` crea entradas nuevas**: no sobrescribe por `sync_id`. Cada import genera nuevos IDs locales. Esto es útil para merge.
- **`engram delete project --hard` y FKs**: falla si hay observaciones en otro proyecto que referencian sesiones del proyecto a borrar.
- **No hay `rename` de proyecto**: el camino es export → filtrar JSON → import → soft-delete → UPDATE SQL para sesiones.
- **`engram cloud enroll` es obligatorio** antes del primer `sync --cloud`.

## Caso real: cron-insta (junio 2026)

| Paso | Resultado |
|------|-----------|
| Import DB antigua | 73 sesiones, 338 obs, 676 prompts |
| Filtrar cronista → cron-insta | 18 sesiones, 170 obs, 380 prompts |
| Soft-delete cronista | Elimina obs/prompts duplicados, sesiones quedan |
| SQL UPDATE sessions | 18 sesiones movidas a cron-insta |
| Cloud enroll + sync | 570 mutations subidas |
| **Estado final** | 170 obs, 19 sesiones, 381 prompts ☁️ |
