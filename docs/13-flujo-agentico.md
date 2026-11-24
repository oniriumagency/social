# 13 - Flujo de Trabajo con Agentes

Cómo se construye este proyecto con herramientas de programación agéntica (Claude Code, Antigravity u otras). Las reglas obligatorias están resumidas en `CLAUDE.md` y `ANTIGRAVITY.md`; este documento explica el procedimiento completo.

## 1. Roles

| Rol | Quién | Responsabilidad |
| :-- | :-- | :-- |
| **Responsable humano** | Magne | Decide producto y prioridades, aprueba compuertas, aprueba PR, resuelve decisiones ejecutivas, tiene las credenciales. |
| **Agente implementador** | Claude Code o Antigravity | Ejecuta una tarea del plan de principio a fin en una rama, con pruebas, y abre el PR. |
| **Agente revisor** | Una sesión distinta, sin el contexto del implementador | Revisa el PR contra la tarea, los documentos y las invariantes. |
| **Auditor externo** | Terceros | Compuertas G2, G3 y G4. |

Un agente nunca aprueba ni fusiona su propio trabajo.

## 2. Fuentes de verdad y precedencia

1. **Invariantes** de `CLAUDE.md` / `ANTIGRAVITY.md`. No se negocian en una tarea.
2. **ADR aceptados**, en `docs/adr/registro-de-decisiones.md`.
3. **Documentos numerados** `01` a `16`.
4. **Plan** (`12`) y **estado** (`docs/estado/PROGRESO.md`).
5. El código.

Si dos fuentes se contradicen, gana la de número menor en esta lista y **el agente se detiene y lo informa**. No elige por su cuenta.

Si el código y un documento difieren, uno de los dos está mal. Se corrige en el mismo PR o se abre una incidencia; nunca se deja pasar.

## 3. Ciclo de una tarea

```
1. ELEGIR     Tomar la siguiente tarea "pendiente" con dependencias cumplidas en PROGRESO.md.
              Marcarla "en curso" con fecha y herramienta.
2. LEER       La fila de la tarea en 12, los documentos que cita, los ADR relacionados
              y el código que va a tocar. Nada más: no cargar todo el repositorio.
3. PLANIFICAR Escribir un plan corto: archivos, pruebas, riesgos, dudas.
              Si toca una zona sensible o hay una duda de las de §6, parar y preguntar.
4. PROBAR     Escribir primero las pruebas que expresan el criterio de aceptación.
5. IMPLEMENTAR El cambio mínimo que cumple la tarea. Sin extras.
6. VERIFICAR  Ejecutar `pnpm check`. Corregir hasta que pase. Ejecutar lo que se construyó
              y comprobar que funciona de verdad, no solo que compila.
7. DOCUMENTAR Actualizar los documentos, esquemas y vectores afectados. ADR si hubo decisión.
8. ENTREGAR   Commit, PR con la plantilla, marcar "en revisión" en PROGRESO.md.
9. CERRAR     Tras la fusión, marcar "hecha" con el enlace al PR.
```

Una tarea, una rama, un PR. Si aparece trabajo no previsto, se anota como tarea nueva en `12` y en `PROGRESO.md`; no se hace dentro del PR actual.

## 4. Definición de hecho

Una tarea está hecha solo si se cumple **todo**:

- [ ] El criterio de aceptación de la tarea se cumple y hay una prueba automática que lo demuestra.
- [ ] `pnpm check` pasa en local.
- [ ] CI pasa en Linux y Windows.
- [ ] No se desactivó, borró ni debilitó ninguna prueba existente.
- [ ] Ninguna invariante se viola.
- [ ] Los documentos afectados están actualizados en el mismo PR.
- [ ] Si se cambió el protocolo: esquemas, vectores dorados y `06` actualizados.
- [ ] Si se tomó una decisión de arquitectura: hay ADR.
- [ ] Sin `TODO` sin incidencia, sin código muerto, sin registros de depuración.
- [ ] Textos de interfaz en `packages/i18n`, en español e inglés.
- [ ] El PR describe cómo se probó y qué quedó fuera.
- [ ] `PROGRESO.md` actualizado.

## 5. Gestión del contexto entre sesiones

Los agentes no recuerdan sesiones anteriores. El repositorio es la memoria.

| Necesidad | Dónde se guarda |
| :-- | :-- |
| Qué está hecho y qué sigue | `docs/estado/PROGRESO.md` |
| Por qué se decidió algo | `docs/adr/registro-de-decisiones.md` |
| Resultados de investigación | `docs/spikes/` |
| Notas de traspaso de una tarea a medias | Sección "Traspaso" de la tarea en `PROGRESO.md` |
| Cómo operar algo | `docs/runbooks/` |

Reglas:

1. Al empezar una sesión, el agente lee `PROGRESO.md` antes que nada.
2. Si una sesión termina con la tarea a medias, deja una nota de traspaso: qué hizo, qué falta, qué probó, qué le preocupa.
3. No se crean documentos de planificación o análisis sueltos fuera de estas carpetas.
4. Dos herramientas pueden trabajar a la vez solo en tareas distintas y en ramas distintas.

## 6. Cuándo detenerse y preguntar

El agente **se detiene y pregunta al responsable humano** antes de continuar si:

1. La tarea exige tocar una zona sensible de forma no prevista en su descripción.
2. Haría falta una dependencia nueva en `social-crypto`, `social-core`, registro, ledger o red.
3. Haría falta cambiar el protocolo, un esquema de base de datos ya desplegado, o una invariante.
4. Dos documentos se contradicen.
5. El criterio de aceptación no se puede cumplir tal como está escrito.
6. Para que pase una prueba, la única salida que ve es desactivarla o debilitarla.
7. Necesita un secreto, una credencial o acceso a un entorno que no sea local.
8. Encuentra lo que parece una vulnerabilidad.
9. Lleva tres intentos fallidos sobre el mismo problema.
10. La tarea requiere una decisión marcada como pendiente en `15`.

Preguntar bien: una sola pregunta concreta, con las opciones, la recomendación y el motivo.

El agente **no pregunta** por lo que puede resolver leyendo el repositorio, la documentación oficial o ejecutando una prueba.

## 7. Prácticas que hacen fiable el trabajo agéntico

1. **Verificar, no recordar.** Antes de usar una biblioteca, leer su documentación vigente o su código. Las interfaces cambian y la memoria del modelo puede estar desactualizada.
2. **Pruebas como especificación.** El criterio de aceptación se convierte en prueba antes de implementar.
3. **Cambios pequeños.** PR de menos de unas 600 líneas. Más grande significa dividir la tarea.
4. **Ejecutar de verdad.** Compilar no es probar. Arrancar el servicio, llamar al endpoint, abrir la pantalla.
5. **Reportar con honestidad.** Si algo falla, se dice y se adjunta la salida. Si un paso no se hizo, se dice. Nunca "debería funcionar".
6. **Revisión con ojos frescos.** El revisor recibe el diff y la tarea, no la conversación del implementador.
7. **Sin atajos silenciosos.** Un simulacro (mock) donde la tarea pedía la pieza real, un valor fijo donde debía haber un cálculo, o una excepción tragada son defectos graves.
8. **Deshacer es barato; ocultar es caro.** Ante la duda, una rama nueva y un PR en borrador.

## 8. Plantillas de instrucción

**Empezar una tarea**

```
Ejecuta la tarea F1-T08 siguiendo docs/13-flujo-agentico.md §3.
Lee primero docs/estado/PROGRESO.md y la fila de la tarea en docs/12-plan-de-desarrollo.md.
Antes de escribir código, muéstrame el plan y las pruebas que vas a crear.
```

**Revisar un PR**

```
Revisa el PR de la rama tarea/F1-T08-feed-por-zona como revisor independiente.
Comprueba: criterio de aceptación de la tarea, definición de hecho (13 §4),
invariantes de CLAUDE.md, y coherencia con los documentos que cita la tarea.
Lista hallazgos por severidad con archivo y línea. No corrijas nada.
```

**Refinar una fase**

```
Ejecuta F2-T00. Lee los informes de docs/spikes/, el estado en PROGRESO.md y los
documentos 05 y 12. Propón el desglose actualizado de F2 en tareas de un PR, con
criterios de aceptación verificables. No implementes nada. Espera mi aprobación.
```

**Ejecutar un spike**

```
Ejecuta el spike F0-S3. Es investigación con prototipo desechable: el entregable es el
informe docs/spikes/S3-zonas.md con datos medidos, no código de producción.
Responde cada pregunta de la tarea con evidencia. Termina con recomendación y borrador de ADR.
```

## 9. Compuertas

Al terminar una fase, el agente prepara un **informe de compuerta** en `docs/estado/G<n>-informe.md`:

1. Tareas completadas con enlaces.
2. Criterios de la compuerta, uno por uno, con evidencia.
3. Hallazgos de seguridad abiertos.
4. Decisiones pendientes que bloquean la fase siguiente.
5. Riesgos nuevos detectados.

El responsable humano aprueba o devuelve. Sin aprobación escrita en ese archivo, no empieza la fase siguiente.
