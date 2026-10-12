# Draft PR (кратко)

Title: GROOT: backend adapter (gitoxide integration scaffold)

Краткое описание:
- Добавляет слой адаптера (backend trait) для «шва» между grit и gitoxide.
- Оставляет legacy-реализацию как fallback и добавляет заготовку для GitoxideAdapter.
- Включает базовые smoke‑тесты для discovery/index/object и документ миграции.

Ключевые изменения:
- backend trait + LegacyAdapter + GitoxideAdapter (stub)
- docs/migration_plan.md — детальный план миграции
- CI: запуск тестов в матрице (legacy)
- совместные smoke‑тесты в `crates/groot-compat`

Acceptance criteria (кратко):
- `cargo build --workspace` и `cargo test --workspace` проходят в CI
- legacy backend остаётся доступен
- gitoxide adapter есть как цель миграции (stub)

Следующие шаги:
1. реализовать `GitoxideAdapter::discover` (вызов gix::discover/open)
2. реализовать чтение объектов через gix-odb
3. мигрировать индекс на gix-index
4. покрыть edge-cases из grit (gitfile, commondir, GIT_DIR/WORK_TREE)

Как открыть PR: можно создать Draft PR из ветки `feature/groot-backend-adapter` в `main` через GitHub UI или:

```bash
gh pr create --base main --head feature/groot-backend-adapter --title "GROOT: backend adapter (gitoxide)" --body-file docs/pr_draft_short.md --draft
```

