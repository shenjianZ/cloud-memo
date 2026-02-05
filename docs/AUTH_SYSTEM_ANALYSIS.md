# 认证系统架构分析

## 📋 目录

1. [系统架构](#系统架构)
2. [后端实现](#后端实现)
3. [前端实现](#前端实现)
4. [数据流程](#数据流程)
5. [UI 现状](#ui-现状)
6. [改进建议](#改进建议)

---

## 系统架构

### 整体架构图

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 前端                           │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐  │
│  │   UI Layer  │  │AuthStore    │  │  authApi.ts  │  │
│  │ (React)     │  │(Zustand)    │  │(Tauri API)  │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬───────┘  │
│         │                │                 │           │
└─────────┼────────────────┼─────────────────┼───────────┘
          │                │                 │
          ▼                ▼                 ▼
┌─────────────────────────────────────────────────────────┐
│                   Tauri Commands Layer                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │  - login()    - register()   - logout()          │  │
│  │  - get_current_user()  - is_authenticated()     │  │
│  └───────────────────┬──────────────────────────────┘  │
└──────────────────────┼──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                   Service Layer                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │           AuthService (auth_service.rs)          │  │
│  │  - login()    - register()   - logout()          │  │
│  │  - 加密存储 token                                   │  │
│  │  - HTTP 请求到服务器                               │  │
│  └───────────────────┬──────────────────────────────┘  │
└──────────────────────┼──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                数据层 (数据库 + 网络)                     │
│  ┌─────────────────┐  ┌──────────────────────────┐     │
│  │  user_auth 表   │  │    HTTP Server            │     │
│  │  (加密存储)      │  │  (note-sync-server)      │     │
│  └─────────────────┘  └──────────────────────────┘     │
└─────────────────────────────────────────────────────────┘
```

---

## 后端实现

### 1. 数据模型 (`src-tauri/src/models/auth.rs`)

#### 核心数据结构

```rust
// 登录/注册请求
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub server_url: String,
}

// 认证响应
pub struct AuthResponse {
    pub token: String,           // JWT 访问令牌
    pub refresh_token: Option<String>,
    pub user_id: String,
    pub email: String,
    pub expires_at: i64,         // Unix 时间戳（秒）
}

// 用户信息
pub struct User {
    pub id: String,
    pub email: String,
    pub server_url: String,
    pub device_id: String,
    pub last_sync_at: Option<i64>,
}
```

---

### 2. 数据库 Schema

```sql
CREATE TABLE IF NOT EXISTS user_auth (
    id INTEGER PRIMARY KEY,
    user_id TEXT NOT NULL,
    server_url TEXT NOT NULL,
    email TEXT NOT NULL,
    access_token_encrypted TEXT NOT NULL,  -- AES-256 加密存储
    refresh_token_encrypted TEXT,
    token_expires_at INTEGER,
    device_id TEXT NOT NULL,
    last_sync_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

**安全特性**：
- ✅ Token 使用 AES-256-GCM 加密存储
- ✅ 每个设备有唯一的 device_id
- ✅ 密码不在本地存储

---

### 3. AuthService (`src-tauri/src/services/auth_service.rs`)

#### 核心功能

##### 3.1 用户登录

```rust
pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse> {
    // 1. 生成或获取设备 ID
    let device_id = self.get_or_create_device_id()?;

    // 2. 发送 HTTP POST 请求到服务器
    let url = format!("{}/auth/login", server_url);
    let response = self.client
        .post(&url)
        .json(&json!({
            "email": req.email,
            "password": req.password
        }))
        .send()
        .await?;

    // 3. 解析响应
    let token = server_response["token"].as_str()?;
    let user_id = server_response["user_id"].as_str()?;

    // 4. 加密并存储 token
    self.save_user_auth(
        &req.server_url,
        &req.email,
        &token,
        &token,
        expires_at,
        &device_id,
    )?;

    Ok(AuthResponse { ... })
}
```

##### 3.2 Token 加密存储

```rust
/// 加密 token 并保存到数据库
fn save_user_auth(
    &self,
    server_url: &str,
    email: &str,
    access_token: &str,
    refresh_token: &str,
    expires_at: i64,
    device_id: &str,
) -> Result<()> {
    // 1. 生成密钥（固定密钥，实际应该使用设备派生）
    let key = b"your-32-byte-secret-key-1234567890ab";  // ⚠️ 需要改进

    // 2. 生成随机 nonce
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // 3. 加密 token
    let ciphertext = cipher.encrypt(&nonce, access_token.as_bytes())?;

    // 4. Base64 编码
    let encrypted_token = general_purpose::STANDARD.encode(ciphertext);

    // 5. 保存到数据库
    conn.execute(
        "INSERT OR REPLACE INTO user_auth (...)",
        params![...]
    )?;

    Ok(())
}
```

**安全改进点**：
- ⚠️ **密钥硬编码**：应该使用设备指纹派生密钥
- ✅ 使用 AES-256-GCM 认证加密
- ✅ 每次加密使用随机 nonce

##### 3.3 用户注册

```rust
pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse> {
    let device_id = self.get_or_create_device_id()?;
    let url = format!("{}/auth/register", req.server_url);

    let response = self.client
        .post(&url)
        .json(&json!({
            "email": req.email,
            "password": req.password,
            "device_id": &device_id,
        }))
        .send()
        .await?;

    // 解析响应并保存 token（与登录相同）
    // ...
}
```

##### 3.4 检查认证状态

```rust
pub fn is_authenticated(&self) -> Result<bool> {
    let conn = self.pool.get()?;
    let exists = conn.execute(
        "SELECT 1 FROM user_auth WHERE id = 1",
        [],
    )?;

    Ok(exists > 0)
}
```

---

### 4. Tauri Commands (`src-tauri/src/commands/auth.rs`)

暴露给前端的 API：

```rust
#[tauri::command]
pub async fn login(
    req: LoginRequest,
    service: AuthSvc<'_>,
) -> std::result::Result<AuthResponse, String> {
    service.login(req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn register(
    req: RegisterRequest,
    service: AuthSvc<'_>,
) -> std::result::Result<AuthResponse, String> {
    service.register(req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn logout(service: AuthSvc<'_>) -> std::result::Result<(), String> {
    service.logout().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_user(
    service: AuthSvc<'_>,
) -> std::result::Result<User, String> {
    service.get_current_user().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_authenticated(
    service: AuthSvc<'_>,
) -> std::result::Result<bool, String> {
    service.is_authenticated().map_err(|e| e.to_string())
}
```

---

## 前端实现

### 1. API 层 (`src/services/authApi.ts`)

```typescript
import { invoke } from '@tauri-apps/api/core';

// 登录
export async function login(req: LoginRequest): Promise<AuthResponse> {
  return invoke('login', { req });
}

// 注册
export async function register(req: RegisterRequest): Promise<AuthResponse> {
  return invoke('register', { req });
}

// 登出
export async function logout(): Promise<void> {
  return invoke('logout');
}

// 获取当前用户
export async function getCurrentUser(): Promise<User> {
  return invoke('get_current_user');
}

// 检查认证状态
export async function isAuthenticated(): Promise<boolean> {
  return invoke('is_authenticated');
}
```

---

### 2. Store 层 (`src/store/authStore.ts`)

使用 Zustand + persist 实现状态管理和持久化：

```typescript
interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null

  // Actions
  login: (email, password, serverUrl) => Promise<void>
  register: (email, password, serverUrl) => Promise<void>
  logout: () => Promise<void>
  checkAuth: () => Promise<void>
  clearError: () => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      login: async (email, password, serverUrl) => {
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.login({ email, password, server_url: serverUrl })
          const user: User = {
            id: response.user_id,
            email: response.email,
            server_url: serverUrl,
            device_id: '',
          }
          set({ user, isAuthenticated: true, isLoading: false })
        } catch (error) {
          set({ error: error.message, isLoading: false })
          throw error
        }
      },

      // ... 其他方法
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)
```

**持久化策略**：
- ✅ 只持久化 `user` 和 `isAuthenticated`
- ❌ 不持久化 `error` 和 `isLoading`（避免刷新后仍显示错误）

---

### 3. UI 层

#### 3.1 当前 UI 状态

**已实现**：
- ✅ `AccountSyncSettings` 组件（在 Settings 页面中）
  - 登录/注册表单
  - 用户信息显示
  - 同步状态
  - 登出按钮

**缺失**：
- ❌ 独立的登录页面
- ❌ 独立的注册页面
- ❌ 个人中心页面
- ❌ 路由守卫（未登录用户重定向）

#### 3.2 AccountSyncSettings 组件分析

**位置**：`src/components/sync/AccountSyncSettings.tsx`

**功能**：

1. **未登录状态**
```tsx
if (!isAuthenticated) {
  return (
    <div>
      {/* 登录/注册切换按钮 */}
      <Button onClick={() => setIsLoginMode(true)}>登录</Button>
      <Button onClick={() => setIsLoginMode(false)}>注册</Button>

      {/* 表单 */}
      <Input placeholder="服务器地址" value={serverUrl} />
      <Input placeholder="邮箱" value={email} />
      <Input type="password" placeholder="密码" value={password} />

      <Button onClick={handleAuth}>
        {isLoginMode ? '登录' : '注册'}
      </Button>
    </div>
  )
}
```

2. **已登录状态**
```tsx
return (
  <div>
    {/* 用户信息卡片 */}
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-full bg-blue-500">
          <User className="w-5 h-5 text-white" />
        </div>
        <div>
          <p>{user?.email}</p>
          <p>{user?.server_url}</p>
        </div>
      </div>
      <Button onClick={handleLogout}>登出</Button>
    </div>

    {/* 同步状态 */}
    <div className="flex items-center justify-between">
      <span>同步状态: {status}</span>
      <Button onClick={handleSync}>同步</Button>
    </div>

    {/* 自动同步开关 */}
    <Switch checked={isAutoSyncEnabled} onCheckedChange={setAutoSync} />
  </div>
)
```

---

## 数据流程

### 1. 登录流程

```
用户输入
    ↓
AccountSyncSettings (UI)
    ↓
authStore.login()
    ↓
authApi.login()
    ↓
Tauri invoke('login')
    ↓
AuthService.login()
    ↓
HTTP POST {server_url}/auth/login
    ↓
Server 返回 JWT token
    ↓
AES-256-GCM 加密 token
    ↓
存储到 user_auth 表
    ↓
更新 AuthStore
    ↓
UI 更新
```

### 2. 认证检查流程

```
应用启动
    ↓
AuthStore.checkAuth()
    ↓
authApi.isAuthenticated()
    ↓
Tauri invoke('is_authenticated')
    ↓
AuthService.is_authenticated()
    ↓
查询 user_auth 表
    ↓
返回 true/false
    ↓
更新 AuthStore 状态
```

---

## UI 现状

### 当前路由结构 (`src/routes.tsx`)

```tsx
<Routes>
  <Route path="/" element={<MainLayout />}>
    <Route index element={<Home />} />
    <Route path="editor/:noteId" element={<Editor />} />
    <Route path="notes" element={<AllNotes />} />
    <Route path="favorites" element={<Favorites />} />
    <Route path="trash" element={<Trash />} />
    <Route path="settings" element={<Settings />} />  ← 认证在这里
    <Route path="*" element={<Navigate to="/" replace />} />
  </Route>
</Routes>
```

### 问题分析

#### 1. 没有 `/*` 路由守卫

**当前行为**：
- 用户可以直接访问任何页面
- 不需要登录即可使用应用

**预期行为**：
- 未登录用户访问首页时，应该看到登录提示
- 或者重定向到登录页面

#### 2. 没有独立的个人中心页面

**当前行为**：
- 用户信息分散在 Settings 页面中
- 没有专门的个人信息管理页面

**预期行为**：
- 应该有 `/profile` 或 `/account` 路由
- 显示用户详细信息
- 提供更多管理选项（修改密码、删除账户等）

#### 3. 登录/注册入口不明显

**当前行为**：
- 登录表单隐藏在 Settings → "账户与同步"卡片中
- 用户需要主动进入设置才能登录

**预期行为**：
- 应该有明显的登录入口
- 或者在首次启动时引导用户登录

---

## 改进建议

### 优先级 P0（高优先级）

#### 1. 添加路由守卫

**实现方案**：

```tsx
// src/components/ProtectedRoute.tsx
import { Navigate } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore()

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  return <>{children}</>
}
```

**使用**：

```tsx
// src/routes.tsx
<Route path="/" element={<MainLayout />}>
  <Route
    index
    element={
      <ProtectedRoute>
        <Home />
      </ProtectedRoute>
    }
  />
  {/* 其他路由... */}
</Route>
```

#### 2. 创建独立的登录/注册页面

**文件结构**：
```
src/pages/
  ├── Login.tsx       ← 登录页面
  ├── Register.tsx    ← 注册页面
  └── Profile.tsx     ← 个人中心
```

**登录页面示例**：

```tsx
// src/pages/Login.tsx
export default function Login() {
  const { login, isLoading } = useAuthStore()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [serverUrl, setServerUrl] = useState('http://localhost:3000')

  const handleLogin = async () => {
    try {
      await login(email, password, serverUrl)
      navigate('/')  // 登录成功后跳转首页
    } catch (error) {
      toast.error('登录失败')
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="max-w-md w-full">
        <h1>登录到笔记应用</h1>
        <Input value={email} onChange={(e) => setEmail(e.target.value)} />
        {/* ... */}
        <Button onClick={handleLogin}>登录</Button>
      </div>
    </div>
  )
}
```

#### 3. 添加路由配置

```tsx
// src/routes.tsx
<Routes>
  {/* 公开路由 */}
  <Route path="/login" element={<Login />} />
  <Route path="/register" element={<Register />} />

  {/* 受保护路由 */}
  <Route path="/" element={<MainLayout />}>
    <Route
      index
      element={<ProtectedRoute><Home /></ProtectedRoute>}
    />
    {/* ... */}
  </Route>
</Routes>
```

---

### 优先级 P1（中优先级）

#### 4. 创建个人中心页面

```tsx
// src/pages/Profile.tsx
export default function Profile() {
  const { user, logout } = useAuthStore()

  return (
    <div className="container max-w-2xl mx-auto py-8">
      <h1>个人中心</h1>

      {/* 用户信息卡片 */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-4">
            <Avatar className="w-16 h-16">
              <AvatarFallback>{user?.email[0].toUpperCase()}</AvatarFallback>
            </Avatar>
            <div>
              <h2>{user?.email}</h2>
              <p className="text-muted-foreground">{user?.server_url}</p>
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* 账户设置 */}
      <Card>
        <CardHeader>
          <h3>账户设置</h3>
        </CardHeader>
        <CardContent>
          <Button variant="outline">修改密码</Button>
          <Button variant="destructive">删除账户</Button>
        </CardContent>
      </Card>

      {/* 同步设置 */}
      <AccountSyncSettings />
    </div>
  )
}
```

#### 5. 添加侧边栏用户信息

在 `Sidebar` 组件底部添加用户头像和信息：

```tsx
// src/components/layout/Sidebar.tsx
<div className="p-4 border-t">
  {isAuthenticated ? (
    <div className="flex items-center gap-3">
      <Avatar className="w-8 h-8">
        <AvatarFallback>{user?.email[0].toUpperCase()}</AvatarFallback>
      </Avatar>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium truncate">{user?.email}</p>
        <p className="text-xs text-muted-foreground truncate">已同步</p>
      </div>
      <Button size="sm" variant="ghost" onClick={handleLogout}>
        <LogOut className="w-4 h-4" />
      </Button>
    </div>
  ) : (
    <Button onClick={() => navigate('/settings')}>
      <Cloud className="w-4 h-4 mr-2" />
      登录
    </Button>
  )}
</div>
```

---

### 优先级 P2（低优先级）

#### 6. 优化 token 加密

**当前问题**：
```rust
// ⚠️ 硬编码密钥
let key = b"your-32-byte-secret-key-1234567890ab";
```

**改进方案**：
```rust
// ✅ 使用设备指纹派生密钥
fn derive_key_from_device() -> [u8; 32] {
    let device_id = get_device_id();
    let salt = b"note-app-key-derivation";

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        device_id.as_bytes(),
        salt,
        100000,  // 迭代次数
        &mut key
    );

    key
}
```

#### 7. 添加 Token 刷新机制

**当前问题**：
- Token 过期后需要重新登录
- 没有自动刷新机制

**改进方案**：
```rust
pub async fn refresh_token(&self) -> Result<TokenResponse> {
    let encrypted_refresh_token = self.get_refresh_token_from_db()?;
    let refresh_token = self.decrypt_token(&encrypted_refresh_token)?;

    let response = self.client
        .post(&format!("{}/auth/refresh", server_url))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await?;

    // 更新数据库中的 token
    // ...
}
```

---

## 总结

### 已完成 ✅

1. ✅ 后端认证服务（登录、注册、登出）
2. ✅ Token 加密存储（AES-256-GCM）
3. ✅ 前端 Store（Zustand + persist）
4. ✅ 基础 UI（AccountSyncSettings 组件）

### 待完成 ⚠️

1. ⚠️ 独立的登录/注册页面
2. ⚠️ 个人中心页面
3. ⚠️ 路由守卫
4. ⚠️ Token 刷新机制
5. ⚠️ 优化密钥派生
6. ⚠️ 侧边栏用户信息

### 架构优势 👍

1. ✅ 三层架构清晰（Commands → Service → Repository）
2. ✅ Token 加密存储
3. ✅ 状态持久化（Zustand persist）
4. ✅ TypeScript 类型安全

### 安全改进点 🔒

1. ⚠️ 使用设备指纹派生加密密钥
2. ⚠️ 添加 Token 刷新机制
3. ⚠️ 实现登录过期检测
4. ⚠️ 添加请求签名验证

---

**文档生成时间**：2026-02-04
**版本**：v1.0
