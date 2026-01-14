# Troubleshooting

This document covers common issues and solutions for the Web admin panel, along with security recommendations.

## Common Issues

### Login Failed

```bash
# Check if ADMIN_TOKEN is correctly configured
echo $ADMIN_TOKEN

# Check API address configuration
cat admin-panel/.env.local

# View browser console errors
```

**Possible causes**:

- `ADMIN_TOKEN` not configured or incorrect
- Backend service not running
- API address misconfigured
- CORS configuration issues

### Build Failed

```bash
# Clean dependencies and reinstall
rm -rf node_modules bun.lock
bun install

# Check Bun version
bun --version
```

**Possible causes**:

- Dependency version conflicts
- Bun version too old
- Network issues preventing dependency download

### Style Issues

```bash
# Clear Vite cache
rm -rf admin-panel/.vite
bun dev
```

**Possible causes**:

- Stale Vite cache
- Tailwind CSS configuration issues
- Browser cache

### Blank Page

**Possible causes**:

- JavaScript errors, check browser console
- Router configuration issues
- Environment variables not properly injected

### API Request Failed

**Possible causes**:

- Backend service not running
- CORS configuration issues
- Token expired or invalid
- Network connection problems

## Security Recommendations

1. **Strong Password**: Use sufficiently complex `ADMIN_TOKEN`
2. **HTTPS**: Production must enable HTTPS
3. **Path Isolation**: Consider using non-default `FRONTEND_ROUTE_PREFIX`
4. **Network Isolation**: Only expose admin panel in trusted networks
5. **Regular Updates**: Keep dependencies updated for security fixes

## Getting Help

If the above methods don't solve your problem:

1. Check [GitHub Issues](https://github.com/AptS-1547/shortlinker/issues) for similar problems
2. Submit a new Issue with:
   - Error message screenshots
   - Browser console logs
   - Backend logs
   - Environment info (OS, browser version, Bun version)

## Related Links

- 📖 [Feature Overview](./index)
- 🛠️ [Development Guide](./development)
