## Tarea

<!-- ID del plan (docs/12), por ejemplo F1-T08, y enlace a su fila. Un PR por tarea. -->

## Qué cambia

## Cómo se probó

<!-- Órdenes ejecutadas y resultado. `pnpm check` es obligatoria. Si algo se probó a mano, cómo. -->

- [ ] `pnpm check` en verde en local
- [ ] Lo construido se ejecutó de verdad, no solo se compiló

## Qué quedó fuera

## Lista de seguridad (docs/10, CLAUDE.md §4)

- [ ] Ningún dato de documento ni biométrico sale del dispositivo ni llega a disco o registros (I-1)
- [ ] Ninguna coordenada cruza el límite del dispositivo; solo viaja el identificador de zona (I-2)
- [ ] Ninguna llave privada ni secreto aparece en registros, errores o estado de interfaz (I-3)
- [ ] Ninguna IP se guarda ni se registra junto a un identificador (I-4)
- [ ] No se relacionan cuenta, billetera, nodo e inscripción (I-5)
- [ ] Nada de terceros en los clientes (I-6)
- [ ] La criptografía pasa solo por `social-crypto`; nada de cripto en TypeScript (I-8)
- [ ] Todo evento se valida por completo antes de guardarse, reenviarse o mostrarse (I-9)
- [ ] Sin secretos en el diff, los registros ni la salida (I-10)
- [ ] Ninguna prueba desactivada, borrada o debilitada (I-13)
- [ ] Nada de una fase futura ni fuera del alcance; verificarse nunca se premia (I-14)
- [ ] Si toca una zona sensible (I-15), lo indico aquí y pido revisión humana

## Documentos actualizados

- [ ] `docs/` afectados
- [ ] Esquemas y vectores dorados (`pnpm spec:gen`, `pnpm vectors:gen`) si cambia el protocolo, con ADR
- [ ] `docs/estado/PROGRESO.md`
- [ ] Dependencias nuevas justificadas aquí
