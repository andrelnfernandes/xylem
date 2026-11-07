# Almacenamiento en la nube (rclone)

Xylem no tiene un cliente propio para Google Drive ni S3, pero no lo necesita:
[rclone](https://rclone.org) los monta (junto con unos 70 servicios más:
OneDrive, Dropbox, Backblaze B2, Cloudflare R2, MinIO...) como una carpeta, y
Xylem trabaja sobre esa carpeta como sobre cualquier otra. Allí funciona todo:
registro de directorios, Branch, marcas, el visor, Copiar y Mover en ambas
direcciones, Renombrar, crear directorios y Eliminar.

## Por qué rclone

Elegimos trabajar con rclone en lugar de hablar directamente con Google Drive y
Amazon S3. Tener código propio para cada servicio sería un lugar más donde
cometer errores, y Google Drive traería además las políticas de Google: una
aplicación que lee todos los archivos de alguien requiere la verificación de
Google y una auditoría de seguridad anual, o de lo contrario muestra un aviso de
"aplicación no verificada" y queda limitada a unos pocos usuarios. Para S3, la
biblioteca oficial es muy grande y añadiría mucha complejidad, algo que Xylem,
por diseño, evita siempre que puede. rclone ya hace todo esto bien y para muchos
servicios a la vez.

## Configuración inicial (una sola vez)

Instala rclone (`apt install rclone`, `dnf install rclone`, `brew install
rclone`, o desde [rclone.org](https://rclone.org/downloads/)) y crea un remoto
con `rclone config`. Le das un nombre, que es lo que va antes de los dos puntos
más abajo.

- **Google Drive:** tipo de almacenamiento `drive`; las opciones predeterminadas
  valen. rclone abre el navegador para que inicies sesión con Google y des
  permiso. En una máquina sin navegador (por SSH), responde *no* a la pregunta
  del navegador: rclone muestra un comando para ejecutar con `rclone authorize`
  en una máquina que sí tenga navegador.
- **Amazon S3:** tipo de almacenamiento `s3`, proveedor `AWS`, y luego la clave
  de acceso y el secreto, o bien *env_auth* para usar lo que ya usan las
  herramientas de AWS (`~/.aws/credentials`, variables de entorno, el rol de la
  instancia). Otros servicios compatibles con S3 (R2, MinIO, Wasabi...) aparecen
  como otros proveedores en la misma lista.

`rclone lsd gdrive:` (o `rclone lsd s3:`) lista las carpetas superiores (los
buckets, en S3): si funciona, el remoto está listo.

## Montar y abrir

En Linux, con un remoto llamado `gdrive`:

```bash
mkdir -p ~/gdrive
rclone mount gdrive: ~/gdrive --vfs-cache-mode writes --daemon
xy ~/gdrive
```

Para S3, igual con `s3:` (todos los buckets) o `s3:mi-bucket` (uno de ellos).
`fusermount -u ~/gdrive` desmonta la carpeta.

- `--vfs-cache-mode writes` guarda una copia local de los archivos que se están
  escribiendo, algo que necesitan los editores (`E`) y algunos programas.
- `--dir-cache-time 1h` conserva las listas de directorios durante más tiempo
  que los 5 minutos predeterminados: volver a leer (F3) es más rápido, pero los
  cambios hechos desde otro lugar tardan más en aparecer.

En **macOS**, `rclone mount` necesita [macFUSE](https://osxfuse.github.io);
`rclone nfsmount gdrive: ~/gdrive` no necesita nada adicional. En **Windows**,
necesita [WinFsp](https://winfsp.dev) y se monta en una letra de unidad:
`rclone mount gdrive: X:`, y luego `xy X:\`.

Para tener la carpeta a mano, guárdala como [marcador](bookmarks.md) (Alt+B).

En Linux, Xylem detecta las carpetas montadas por rclone: el panel DISK muestra
el servicio (Google Drive, Amazon S3, OneDrive...) y, debajo, el remoto
(`rclone gdrive:`). Info (Alt+I) y Avail (`A`) también los muestran. El servicio
se obtiene del archivo de configuración de rclone
(`~/.config/rclone/rclone.conf` o `$RCLONE_CONFIG`); cuando está cifrado, el
panel indica simplemente rclone.

## Qué cambia allí

**Cuente con lentitud en algunos momentos.** Todo pasa por rclone y por la red del servicio, así
que puede ser bastante más lento que un disco local, por la propia naturaleza de esta
integración. Ver un archivo, abrir un archivo comprimido o leer una carpeta grande puede tardar,
y la pantalla puede no responder hasta que responda el servicio: en general es espera, no un
bloqueo. En Google Drive, vale la pena crear su propio `client_id`: vea la sección sobre el
error 403, más abajo.

- **Todo va por la red.** Cada lectura de directorio es una petición, y un
  servicio como Google Drive las rechaza cuando son demasiadas. Por eso, en una
  carpeta montada, `*`, Branch (`B`), la vista Usage y Compare en una rama piden
  a rclone toda la rama en un solo listado, mucho más rápido, salvo que ya esté
  toda leída; Esc deja el árbol como estaba. Si rclone falla o muestra otras carpetas que el montaje,
  leen carpeta por carpeta. Solo `*` expande el árbol.
- **La pantalla no espera.** El árbol aparece enseguida y se llena a medida que
  llegan las carpetas, también al abrir una (→ o `+`); mientras tanto, la ventana
  de archivos muestra `reading...`. F3 e ir a una ruta
  escrita todavía esperan su lectura.
- **Los cambios hechos en otro lugar** (en el navegador, en el teléfono) solo
  aparecen después de pulsar F3 y una vez que expira la caché de directorios
  (`--dir-cache-time`).
- **Eliminar:** responde `P` (permanentemente) en lugar de `T`: en Linux, la
  papelera de una carpeta montada es una carpeta `.Trash-` (con tu número de
  usuario) en su raíz, de modo que el archivo seguiría en el remoto ocupando
  espacio. Google Drive guarda igualmente en su propia papelera lo que rclone
  elimina; en S3 un archivo eliminado desaparece, a menos que el bucket tenga
  versionado.
- **Los permisos y propietarios** no se conservan: el montaje muestra los mismos
  para todos los archivos y Atributos no puede cambiarlos. Las fechas sí se
  conservan.
- **Git:** la lista de archivos no muestra las letras de estado en una carpeta montada,
  porque `git status` lee todo el repositorio por la red. Ctrl+G sigue funcionando allí.
- **Google Docs, Sheets y Slides** no son archivos normales: rclone los muestra
  exportados (`.docx`, `.xlsx`, `.pptx`), con tamaño 0. `--drive-skip-gdocs` los
  oculta.
- **S3 no tiene directorios reales**, solo nombres con `/` en ellos: un
  directorio vacío puede desaparecer al desmontar, y renombrar o mover un
  directorio copia y elimina cada archivo que contiene.

## Cuando el servicio rechaza: error 403

Google Drive limita cuántas peticiones acepta por minuto. En un Drive saturado de
archivos y carpetas pequeños (copias de proyectos, `node_modules`, extensiones de
editor), **los errores 403 son comunes**: `Quota exceeded`, `rateLimitExceeded`.
rclone espera y vuelve a intentarlo, así que todo se vuelve lento, y al final
puede rendirse. Cualquier gestor de archivos que lea carpeta por carpeta a través
del montaje choca con el mismo límite.

Lo que hace Xylem: `*` muestra cuántas peticiones se rechazaron y, tras 100,
pregunta si seguir intentándolo o parar; parar deja el árbol como estaba. Si
rclone se rinde, Xylem lo avisa y no pasa a leer carpeta por carpeta por su
cuenta, porque eso solo se rechazaría más.

Lo que ayuda, empezando por lo primero:

- **Registrar por carpetas.** En vez de `*` en la raíz, usa `*` en carpetas más
  pequeñas, o ábrelas con → o `+`. En un Drive saturado suele ser el mejor camino.
- **Tu propio `client_id`.** Si no creaste uno, rclone usa un cliente compartido
  con sus demás usuarios, y la cuota también. Crear el tuyo es gratis: la
  [guía de rclone](https://rclone.org/drive/#making-your-own-client-id) lo explica.
  Después, vuelve a autorizar con `rclone config`. Los límites de tu cuenta siguen,
  pero el uso de los demás deja de contar contra ti.
- **Guardar los listados más tiempo:** `--dir-cache-time 1h` en `rclone mount`.
- **Quitar lo que se puede recrear.** Menos carpetas son menos peticiones, en
  cualquier programa.

## Encontrar qué llena tu Google Drive

Cuando Google Drive insiste en pedirte un plan de almacenamiento mayor, Xylem
puede mostrarte adónde se va el espacio antes de pagar por más. Monta el Drive,
ve a su raíz, regístralo todo con `*` (en un Drive grande lleva un rato) y elige
Alt+K → **Percent** ([Tamaños de directorios](info.md)): cada carpeta muestra
qué porcentaje de la carpeta superior ocupa, con una barra proporcional a la más
grande de sus carpetas hermanas. Sigue las barras más grandes hasta dar con las
copias de seguridad antiguas, los vídeos y los duplicados.

La [vista de uso](usage.md) (Alt+W, en la raíz del Drive) va más allá: muestra
los archivos y carpetas juntos, de mayor a menor, para que un único archivo
enorme destaque igual que una carpeta llena. Y Branch (`B`) lista todos los
archivos de una carpeta completa: [ordenados](sort.md) por tamaño de mayor a
menor (Alt+S, luego `O` para descendente y `S` para tamaño), los más grandes
quedan arriba del todo.

Algunas cosas consumen espacio del plan sin aparecer en el montaje:

- **La papelera del Drive** ocupa espacio hasta que se vacía (o pasan 30 días).
  `rclone about gdrive:` muestra el total, lo usado y lo que hay en la papelera;
  montar con `--drive-trashed-only` muestra únicamente lo que está en la
  papelera.
- **Gmail y Google Fotos** comparten el mismo almacenamiento, y en el montaje
  solo están los archivos de Drive. Si `rclone about` muestra mucho más espacio
  usado del que suma el montaje, es probable que el resto esté ahí.
- **Google Docs, Sheets y Slides** aparecen con tamaño 0, pero los más recientes
  sí cuentan para el almacenamiento.
- Los archivos **compartidos contigo** no cuentan: están en el almacenamiento de
  su propietario.
