# Flutter Folder Structure

```
mobile/lib/
├── main.dart
├── core/
│   ├── constants/app_constants.dart
│   ├── network/api_client.dart          # Dio + JWT refresh
│   ├── router/app_router.dart           # go_router shell
│   ├── storage/secure_storage.dart
│   ├── theme/app_theme.dart
│   ├── theme/theme_controller.dart
│   └── widgets/
│       ├── glass_card.dart
│       └── main_shell.dart
└── features/
    ├── auth/
    │   ├── data/auth_repository.dart
    │   ├── domain/user.dart
    │   └── presentation/
    │       ├── auth_controller.dart
    │       ├── login_screen.dart
    │       └── register_screen.dart
    ├── home/presentation/home_screen.dart
    ├── ipo/presentation/
    │   ├── ipo_providers.dart
    │   ├── ipo_list_screen.dart
    │   └── ipo_detail_screen.dart
    ├── portfolio/presentation/
    │   ├── portfolio_providers.dart
    │   └── portfolio_screen.dart
    ├── journal/presentation/
    │   ├── journal_screen.dart
    │   └── trade_entry_screen.dart
    ├── ai_assistant/presentation/ai_chat_screen.dart
    └── settings/presentation/settings_screen.dart
```

## Clean Architecture mapping

| Feature layer | Role |
|---------------|------|
| `presentation` | Screens, Riverpod UI controllers |
| `domain` | Pure entities / contracts |
| `data` | API repositories implementing domain |

Expand with `domain/repositories/*.dart` interfaces when adding tests doubles.

## Platform bootstrap

After clone, generate native projects once:

```bash
cd mobile && flutter create . --project-name investiq_ai --org ai.investiq
```
