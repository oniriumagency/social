---
name: tarea
description: Empieza una tarea del plan de desarrollo de Social por su ID (por ejemplo F0-T05) siguiendo el ciclo completo del proyecto. Úsala cuando el usuario pida ejecutar, empezar o continuar una tarea del plan.
---

# Ejecutar una tarea del plan

El usuario indica un ID de tarea (por ejemplo `F1-T08`). Si no lo indica, lee `docs/estado/PROGRESO.md`, propón la siguiente tarea pendiente con dependencias cumplidas y espera confirmación.

## Pasos

1. Lee `docs/estado/PROGRESO.md`. Si la tarea está "en curso" por otra herramienta o "bloqueada", detente e informa.
2. Lee la fila de la tarea en `docs/12-plan-de-desarrollo.md` y cada documento que cita. Usa la tabla "Qué leer según lo que toques" de `ANTIGRAVITY.md`.
3. Comprueba que las dependencias de la tarea están "hechas". Si no, detente.
4. Crea la rama `tarea/<ID>-<descripcion-corta>` y marca la tarea "en curso" en `PROGRESO.md` con fecha y herramienta.
5. Presenta un plan corto: archivos a crear o modificar, pruebas que expresan el criterio de aceptación, riesgos y dudas. Si la tarea toca una zona sensible, el protocolo o una migración, **espera aprobación**.
6. Escribe primero las pruebas. Después implementa el cambio mínimo.
7. Ejecuta `pnpm check` hasta que pase. Ejecuta de verdad lo construido y comprueba que funciona.
8. Actualiza documentos, esquemas, vectores y ADR afectados.
9. Repasa la definición de hecho de `docs/13-flujo-agentico.md` §4, punto por punto.
10. Prepara el commit y el PR con la plantilla. Marca la tarea "en revisión".

## Reglas

- Una sola tarea. El trabajo no previsto se anota como tarea nueva; no se hace aquí.
- Si se cumple cualquier condición de "Detente y pregunta", para.
- Si la sesión termina a medias, deja la nota de traspaso en `PROGRESO.md`.
- Informa con honestidad de lo que falló o no se hizo.
