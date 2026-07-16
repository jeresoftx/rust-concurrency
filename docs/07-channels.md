# Channels

> **Curso:** rust-concurrency · **Capítulo:** 07 · **Prerrequisitos:** ownership
> en Rust, `Send`, threads, `Mutex`, `Arc` y cierre de recursos
> **Código:** [`src/channels.rs`](../src/channels.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Un channel permite que un hilo envíe mensajes a otro. En lugar de compartir una
estructura mutable protegida por locks, el productor transfiere propiedad de un
valor y el consumidor lo recibe como dueño. Esto cambia la pregunta central: ya
no es "quién puede tocar este dato compartido", sino "quién posee el siguiente
mensaje".

Este capítulo enseña paso de mensajes, productores, consumidores, canales
acotados y no acotados, presión de regreso o backpressure, cierre del canal,
distribución y concentración de resultados, y comparación contra memoria
compartida. La meta es aprender a diseñar flujos concurrentes donde el
movimiento de datos sea parte visible del modelo.

## Motivación

Supón que tienes un servicio que recibe trabajos, los procesa en varios hilos y
concentra los resultados. Con `Arc<Mutex<Vec<Job>>>`, todos los trabajadores
compiten por la misma cola y tienes que razonar sobre locks, scopes, contención
y cierre. Con un channel, el productor envía trabajos y cada trabajador recibe
propiedad de un trabajo. Cuando todos los productores se cierran, los
consumidores observan el cierre y pueden terminar.

Un channel no elimina la necesidad de diseño. Un canal no acotado puede crecer
sin límite si los productores son más rápidos que los consumidores. Un canal
acotado puede bloquear o rechazar envíos cuando se llena. Esa presión de regreso
es una herramienta de estabilidad, no un accidente.

## Teoría

### Historia

El paso de mensajes aparece en modelos de concurrencia como CSP y en sistemas
tipo actor. La idea es sencilla y profunda: comunicar mediante mensajes evita
muchas formas de estado compartido mutable. En Rust, esta idea encaja con
ownership: enviar un valor por un channel suele mover su propiedad al
consumidor.

La biblioteca estándar ofrece `std::sync::mpsc`: múltiples productores y un
consumidor. También ofrece canales acotados con `sync_channel`, donde la
capacidad finita vuelve observable la presión de regreso.

### Fundamentos

Un channel tiene dos extremos: productor y consumidor. El productor envía
mensajes; el consumidor los recibe en orden FIFO para un mismo canal. Cuando
todos los productores desaparecen, el consumidor puede observar cierre. Cuando
el consumidor desaparece, un productor que intenta enviar recibe el mensaje de
vuelta como error.

Los canales no acotados favorecen ergonomía y rendimiento cuando la tasa de
producción está controlada. Su riesgo es acumular memoria si el consumidor no
alcanza a drenar.

Los canales acotados tienen capacidad fija. Si el canal se llena, `send` puede
bloquear y `try_send` devuelve `Full`. Ese comportamiento propaga presión hacia
atrás: el productor no puede ignorar que el consumidor está saturado.

La distribución reparte trabajo entre varios consumidores o trabajadores. La
concentración reúne resultados de varios productores en un solo punto. Juntas
forman un patrón común para pipelines y pools de trabajo.

La invariante central del capítulo es:

```text
un mensaje enviado por channel pertenece al consumidor que lo recibe; el cierre
del canal también es parte del protocolo, no un caso accidental
```

### Casos de uso

Los channels son útiles en:

- pipelines de ingestión, validación y escritura;
- pools de trabajadores para CPU o E/S bloqueante;
- telemetría y logs desde muchos productores;
- coordinación entre hilos sin compartir estructuras internas;
- distribución y concentración de tareas independientes;
- backpressure explícita en servicios con capacidad limitada.

### Ventajas y limitaciones

Ventajas:

- El movimiento de propiedad reduce estado compartido.
- El cierre del canal da una señal natural de terminación.
- Los canales acotados expresan backpressure.
- Múltiples productores pueden alimentar un consumidor sin diseñar locks.
- El patrón de pool de trabajadores separa envío de trabajo y recolección de
  resultados.

Limitaciones:

- Un canal no acotado puede crecer hasta presionar memoria.
- Un canal acotado puede bloquear si no se drena.
- El orden global con múltiples productores depende del interleaving.
- Un solo consumidor puede convertirse en cuello de botella.
- Los channels síncronos no sustituyen un runtime async cuando el problema es
  concurrencia masiva de E/S.

### Comparación con alternativas

`Mutex` y `Arc<Mutex<T>>` son mejores cuando varios hilos deben modificar una
misma invariante en sitio. Un channel es mejor cuando puedes transferir unidades
de trabajo o eventos.

`Arc` comparte propiedad, pero no define por sí mismo cuándo algo se procesa ni
cómo termina un flujo. Un channel expresa secuencia y cierre.

Las colas lock-free pueden reducir ciertos costos de contención, pero suelen
introducir complejidad de memory ordering y reclamación de memoria. Un channel
seguro de la biblioteca estándar es el punto de partida correcto para la mayoría
de diseños educativos.

Los canales async pertenecen al curso `rust-async`. Aquí usamos channels
síncronos para estudiar las invariantes sin introducir un runtime.

Los diseños tipo actor encapsulan estado y comunican por mensajes. Un channel es
una pieza para construirlos, no todo el modelo actor.

## Diagramas

El diagrama principal vive en
[`diagrams/07-channels.mmd`](../diagrams/07-channels.mmd). Muestra productores,
un canal acotado, backpressure, consumidores y concentración de resultados.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `unbounded_channel` | O(1) | O(1) | O(1) | O(1) inicial |
| `Producer::send` | O(1) amortizado | O(1) amortizado | O(1) más costo interno | O(n) acumulado en cola |
| `Consumer::recv` | O(1) | O(1) más espera | espera no acotada si no hay mensaje | O(1) |
| `bounded_channel` | O(1) | O(1) | O(1) | O(capacidad) |
| `BoundedProducer::send` | O(1) | O(1) más espera | espera no acotada si el canal está lleno | O(1) |
| `BoundedProducer::try_send` | O(1) | O(1) | O(1) | O(1) |
| `BoundedConsumer::recv` | O(1) | O(1) más espera | espera no acotada si no hay mensaje | O(1) |
| `WorkerPool::new` | O(w) | O(w) | O(w) | O(w) |
| `WorkerPool::submit` | O(1) amortizado | O(1) amortizado | O(1) más costo interno | O(j) acumulado |
| `WorkerPool::shutdown` | O(j + w) | O(j + w) | O(j + w) más duración de jobs | O(j) resultados |

`w` es el número de trabajadores y `j` el número de trabajos. La complejidad no
captura toda la historia: el costo real depende de scheduling, tamaño del
mensaje, contención y velocidad relativa entre productores y consumidores.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura debería permitir cambiar capacidad,
tasa de producción y tasa de consumo para ver cuándo aparece backpressure.

## Implementación

La implementación del curso define:

- `unbounded_channel`: canal de múltiples productores y un consumidor;
- `Producer` y `Consumer`: extremos del canal no acotado;
- `bounded_channel`: canal con capacidad fija;
- `BoundedProducer` y `BoundedConsumer`: extremos del canal acotado;
- `SendFailure`: error de envío con devolución del mensaje original;
- `TrySendFailure`: `Full` o `Closed` para envío no bloqueante;
- `ChannelReceiveError`: cierre observado desde el consumidor;
- `WorkerPool`: distribución y concentración educativa basada en channels.

La API evita `unsafe` y dependencias externas. El objetivo no es reemplazar
`std::sync::mpsc`, sino poner nombres pedagógicos sobre las garantías que el
curso necesita explicar.

## Pruebas

Las pruebas cubren:

- orden FIFO en canal no acotado;
- cierre observado por el consumidor;
- múltiples productores alimentando un consumidor;
- backpressure con `try_send` en canal acotado;
- envío fallido cuando el consumidor cerró;
- pool de trabajadores con distribución y concentración de resultados;
- canal de capacidad cero como rendezvous.

## Benchmarks

El benchmark manual vive en
[`benches/channels_bench.rs`](../benches/channels_bench.rs). Compara envío y
recepción en canal no acotado, canal acotado y una alternativa con
`Mutex<Vec<T>>`.

Los resultados no eligen arquitectura por sí solos. Un benchmark de channels
debe leerse junto con la invariante: propiedad transferida, cierre explícito y
backpressure cuando la capacidad es finita.

## Ejercicios

### Ejercicio 1: FIFO básico `[Nivel 1]`

Crea un canal no acotado, envía tres mensajes y recíbelos en orden.

**Entrada/Salida esperada:** los mensajes salen como `first`, `second`, `third`.

<details>
<summary>Pista</summary>
Usa un solo productor para que el orden observado sea el orden de envío.
</details>

### Ejercicio 2: Backpressure acotada `[Nivel 2]`

Crea un canal acotado de capacidad 1. Envía un mensaje con `try_send` y verifica
que el segundo envío devuelve `Full`.

**Entrada/Salida esperada:** el primer envío funciona, el segundo devuelve el
mensaje original dentro de `TrySendFailure::Full`.

<details>
<summary>Pista</summary>
Después de recibir el primer mensaje, el canal vuelve a tener capacidad.
</details>

### Ejercicio 3: Worker pool `[Nivel 3]`

Crea un `WorkerPool` que eleve números al cuadrado, envía cinco trabajos y
recolecta los resultados con `shutdown`.

**Entrada/Salida esperada:** los cuadrados `1, 4, 9, 16, 25`, sin depender del
orden en que terminen los trabajadores.

<details>
<summary>Pista</summary>
Ordena el vector de salida antes de compararlo.
</details>

### Ejercicio 4: Diseño de pipeline `[Nivel 4]`

Diseña un pipeline de tres etapas: ingestión, validación y escritura. Decide
qué canales deben ser acotados, qué capacidad usarías y cómo se comunica el
cierre entre etapas.

<details>
<summary>Pista</summary>
La capacidad de un canal es una decisión de producto: define cuánta cola aceptas
antes de frenar productores.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/channels_fifo_order.rs`](../examples/soluciones/channels_fifo_order.rs)
- [`examples/soluciones/channels_bounded_backpressure.rs`](../examples/soluciones/channels_bounded_backpressure.rs)
- [`examples/soluciones/channels_worker_pool.rs`](../examples/soluciones/channels_worker_pool.rs)

Para el nivel 4, una respuesta sana no dice "todo acotado" ni "todo no acotado"
por costumbre. Nombra tasas esperadas, memoria disponible, costo de rechazar
trabajo y cómo se observa saturación.

## Referencias

- Rust Standard Library: `std::sync::mpsc`.
- Rust Book: fearless concurrency y message passing.
- C. A. R. Hoare, *Communicating Sequential Processes*.
- Mara Bos, *Rust Atomics and Locks*.
