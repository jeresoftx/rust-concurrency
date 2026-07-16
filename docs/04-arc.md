# Arc

> **Curso:** rust-concurrency · **Capítulo:** 04 · **Prerrequisitos:** ownership, borrowing, `Mutex`, `RwLock`, atómicos y threads básicos
> **Código:** [`src/arc.rs`](../src/arc.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

`Arc<T>` significa *Atomically Reference Counted*: ownership compartido con
conteo atómico de referencias. Permite que varios hilos tengan un dueño fuerte
del mismo valor en el heap. Mientras exista al menos un dueño fuerte, el valor
sigue vivo. Cuando el último dueño fuerte cae, el valor se libera.

Este capítulo enseña conteo fuerte, referencias débiles, `Arc<Mutex<T>>`, límites
`Send`/`Sync`, costos de clone/drop y riesgos de ciclos. La idea principal es
directa: `Arc` comparte ownership, no vuelve mutable un dato por arte de magia.
Si necesitas mutar desde varios hilos, la mutabilidad debe estar dentro del
valor, por ejemplo con `Mutex` o `RwLock`.

## Motivación

Imagina una tabla de configuración grande que muchos workers necesitan leer. No
quieres clonar toda la tabla para cada hilo, pero tampoco puedes prestar una
referencia normal si los hilos viven más que el scope actual. `Arc<T>` resuelve
ese problema: cada hilo recibe un dueño compartido y el dato vive mientras algún
hilo lo necesite.

Ahora imagina un log compartido que varios hilos deben modificar. `Arc<Vec<_>>`
no alcanza: compartir ownership no autoriza mutación concurrente. Para eso se
necesita `Arc<Mutex<Vec<_>>>` o un diseño distinto. Esta frontera es crucial
para escribir Rust concurrente claro.

## Teoría

### Historia

El conteo de referencias es una técnica antigua para administrar memoria
compartida. En un solo hilo, Rust ofrece `Rc<T>`. En varios hilos, el conteo debe
actualizarse de forma atómica porque clones y drops pueden ocurrir en núcleos
distintos. Ahí entra `Arc<T>`.

`Arc` no es un recolector de basura general. Es ownership compartido con una
regla precisa: el valor muere cuando el conteo fuerte llega a cero. Las
referencias débiles (`Weak<T>`) permiten observar o enlazar sin mantener vivo el
valor, una herramienta esencial para evitar ciclos.

### Fundamentos

Un `Arc<T>` mantiene dos conteos conceptuales:

- conteo fuerte: dueños que mantienen vivo el valor;
- conteo débil: referencias que pueden intentar recuperar el valor, pero no lo
  mantienen vivo.

Clonar un `Arc` incrementa el conteo fuerte. Crear un `Weak` incrementa el conteo
débil. Hacer `upgrade` sobre un `Weak` devuelve `Some(Arc<T>)` si todavía existe
al menos un dueño fuerte, o `None` si el valor ya fue liberado.

La invariante básica es:

```text
el valor T vive mientras strong_count > 0
```

`Arc<T>` implementa `Send` y `Sync` solo cuando `T` cumple las condiciones
necesarias. Esto evita una confusión común: `Arc<T>` permite compartir ownership,
pero no convierte cualquier `T` en seguro para hilos. Si `T` no puede compartirse
entre hilos, envolverlo en `Arc` no arregla el problema.

### Casos de uso

`Arc` aparece en sistemas concurrentes cuando varios hilos necesitan acceso al
mismo dato:

- configuración inmutable compartida;
- tablas de rutas o catálogos de solo lectura;
- `Arc<Mutex<T>>` para estado mutable pequeño;
- `Arc<RwLock<T>>` para muchas lecturas y pocas escrituras;
- handles compartidos hacia recursos de infraestructura;
- grafos donde `Weak` evita ciclos entre nodos.

### Ventajas y limitaciones

Ventajas:

- Permite ownership compartido entre hilos sin copiar todo el dato.
- Libera el valor automáticamente cuando cae el último dueño fuerte.
- `Weak` evita que relaciones auxiliares extiendan vida accidentalmente.
- Encaja con locks internos cuando se necesita mutabilidad compartida.
- Hace explícito el costo de compartir: clone/drop toca conteos atómicos.

Limitaciones:

- No resuelve por sí mismo la mutabilidad concurrente.
- Puede ocultar ownership global si se usa sin diseño.
- Los ciclos con referencias fuertes producen fugas lógicas.
- Clone/drop tiene costo atómico, mayor que un préstamo normal.
- Un `Arc<Mutex<T>>` mal usado puede centralizar demasiada contención.

### Comparación con alternativas

`Rc<T>` es mejor en un solo hilo porque evita costo atómico. Un préstamo normal
es mejor cuando el lifetime cabe en el scope. Clonar el dato puede ser mejor si
el valor es pequeño y quieres independencia. Un channel puede ser mejor si la
intención es transferir ownership a un worker. Scoped threads pueden evitar
`Arc` cuando los hilos no sobreviven al scope que presta los datos.

`Arc` es una respuesta a vida compartida entre hilos, no una licencia para
convertir todo el programa en estado global.

## Diagramas

El diagrama principal vive en [`diagrams/04-arc.mmd`](../diagrams/04-arc.mmd).
Muestra dueños fuertes apuntando al mismo valor, referencias débiles que no lo
mantienen vivo y el intento de `upgrade`.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `Shared::new` | O(1) + mover `T` | O(1) | O(1) | O(1) adicional |
| `clone_shared` | O(1) | O(1) con costo atómico | O(1) | O(1) |
| `downgrade` | O(1) | O(1) con costo atómico | O(1) | O(1) |
| `with_ref` | O(1) + closure | O(1) + closure | O(1) + closure | O(1) |
| `strong_count` | O(1) | O(1) | O(1) | O(1) |
| `weak_count` | O(1) | O(1) | O(1) | O(1) |
| `SharedWeak::upgrade` | O(1) | O(1) con costo atómico | O(1) | O(1) |
| `observations` | O(1) | O(1) | O(1) | O(1) |

El costo relevante no es asintótico sino de coordinación: clone/drop modifican
conteos atómicos compartidos. En rutas muy calientes, ese costo puede importar.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura podría mostrar conteos fuertes y
débiles cambiando al clonar, degradar, hacer `upgrade` y liberar el último dueño
fuerte.

## Implementación

La implementación del curso define:

- `Shared<T>`: wrapper educativo sobre `Arc<T>`;
- `SharedWeak<T>`: wrapper educativo sobre `Weak<T>`;
- `ArcObservation`: snapshot de conteos fuerte y débil.

El wrapper existe para nombrar operaciones pedagógicas: `clone_shared`,
`downgrade`, `upgrade`, `with_ref`, `strong_count`, `weak_count` y
`observations`. No implementa conteo de referencias desde cero y no usa
`unsafe`.

## Pruebas

Las pruebas cubren:

- conteo fuerte al clonar y soltar dueños;
- `Weak::upgrade` exitoso mientras vive el valor;
- `Weak::upgrade` fallando después del último dueño fuerte;
- lectura compartida inmutable desde varios hilos;
- mutabilidad compartida a través de `Mutex`;
- observaciones disponibles desde una referencia débil.

## Benchmarks

El benchmark manual vive en [`benches/arc_bench.rs`](../benches/arc_bench.rs).
Mide:

- clone/drop de `std::sync::Arc`;
- clone/drop de `Shared`;
- lectura compartida desde varios hilos;
- lectura con datos clonados por hilo.

La comparación no busca coronar una estrategia universal. Sirve para preguntar:
¿quiero compartir el mismo dato o prefiero copias independientes?

## Ejercicios

### Ejercicio 1: Conteo de clones `[Nivel 1]`

Crea un `Shared<&str>`, clónalo una vez y observa `strong_count` antes y después
de soltar el clon.

**Entrada/Salida esperada:** `strong_count` pasa de `2` a `1`.

<details>
<summary>Pista</summary>
Cada dueño fuerte mantiene vivo el mismo valor.
</details>

### Ejercicio 2: Catálogo compartido `[Nivel 2]`

Comparte un catálogo inmutable con varios hilos usando `Shared<Vec<String>>`.
Cada hilo debe leer la cantidad de capítulos.

**Entrada/Salida esperada:** todos los hilos observan la misma longitud.

<details>
<summary>Pista</summary>
Clona el `Shared`, no el `Vec`.
</details>

### Ejercicio 3: Weak y vida del valor `[Nivel 3]`

Crea un `Shared`, genera un `SharedWeak`, verifica que `upgrade` funciona y luego
suelta el último dueño fuerte.

**Entrada/Salida esperada:** `upgrade` devuelve `Some` mientras vive el valor y
`None` después.

<details>
<summary>Pista</summary>
Usa un scope interno para controlar cuándo cae el dueño fuerte.
</details>

### Ejercicio 4: Evitar ciclos `[Nivel 4]`

Diseña un grafo pequeño de nodos donde padres apuntan fuerte a hijos e hijos
apuntan débilmente a padres. Explica por qué usar referencias fuertes en ambas
direcciones filtraría memoria lógica.

<details>
<summary>Pista</summary>
Un ciclo fuerte mantiene `strong_count` mayor que cero aunque el programa ya no
tenga una raíz útil.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/arc_clone_counts.rs`](../examples/soluciones/arc_clone_counts.rs)
- [`examples/soluciones/arc_shared_catalog.rs`](../examples/soluciones/arc_shared_catalog.rs)
- [`examples/soluciones/arc_weak_upgrade.rs`](../examples/soluciones/arc_weak_upgrade.rs)

Para el nivel 4 no hay una única implementación canónica en este capítulo. La
regla de diseño es clara: relaciones de propiedad usan `Arc`; relaciones de
observación o vuelta al padre usan `Weak`. Si ambos lados son fuertes, el ciclo
puede sobrevivir sin una raíz semántica real.

## Referencias

- Rust Standard Library: `std::sync::Arc` y `std::sync::Weak`.
- Rust Book: *Fearless Concurrency* y smart pointers.
- Mara Bos, *Rust Atomics and Locks*.
- Rustonomicon: discusión de `Send`, `Sync` y concurrencia.
