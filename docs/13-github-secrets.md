# GitHub Secrets — Firebase / Google Auth

Firebase client values for **InvestIQ AI** are stored as **GitHub Actions secrets** on  
`saurabhahuja71/InvestIQ-AI` (Settings → Secrets and variables → Actions).

They were set from a machine that had a valid `mobile/config/firebase.dart-define.json` + `.env`.

## Important limitation

GitHub secrets are **write-only**:

- You **cannot** open a secret later and read its value in the UI.
- You can only **use** them in GitHub Actions, or **overwrite** them with a new value.
- For human “look up the key anywhere” storage, also keep a password manager / private note.

## Secret names (no values)

| Secret name | Purpose |
|-------------|---------|
| `FIREBASE_API_KEY` | Web API key |
| `FIREBASE_APP_ID` | Web app ID |
| `FIREBASE_MESSAGING_SENDER_ID` | Sender ID |
| `FIREBASE_PROJECT_ID` | Project ID (also backend JWT audience) |
| `FIREBASE_AUTH_DOMAIN` | Auth domain |
| `FIREBASE_STORAGE_BUCKET` | Storage bucket |
| `FIREBASE_MEASUREMENT_ID` | Analytics measurement ID (optional) |
| `GOOGLE_WEB_CLIENT_ID` | OAuth Web client ID (Flutter `serverClientId`) |
| `GOOGLE_CLIENT_IDS` | Backend audience list (comma-separated) |
| `FIREBASE_CONFIG_JSON` | Full JSON bundle of the above for easy restore in CI |

Empty / not set yet (add after Android app is registered):

- `FIREBASE_ANDROID_API_KEY`
- `FIREBASE_ANDROID_APP_ID`
- `FIREBASE_ANDROID_MESSAGING_SENDER_ID`

## Update secrets from this machine

```bash
# After editing firebase.dart-define.json
cd ~/InvestIQ-AI
printf '%s' "$(jq -r .FIREBASE_API_KEY mobile/config/firebase.dart-define.json)" \
  | gh secret set FIREBASE_API_KEY

# Or whole bundle:
gh secret set FIREBASE_CONFIG_JSON < mobile/config/firebase.dart-define.json
```

## Use in GitHub Actions

```yaml
env:
  FIREBASE_PROJECT_ID: ${{ secrets.FIREBASE_PROJECT_ID }}
  FIREBASE_API_KEY: ${{ secrets.FIREBASE_API_KEY }}
  # ...
```

Or write a dart-define file in CI:

```yaml
- name: Write Firebase dart-defines
  run: echo '${{ secrets.FIREBASE_CONFIG_JSON }}' > mobile/config/firebase.dart-define.json
```

## UI

https://github.com/saurabhahuja71/InvestIQ-AI/settings/secrets/actions
