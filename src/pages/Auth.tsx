import { useState, useEffect } from "react";
import { useAuthStore } from "@/store/authStore";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { toast } from "sonner";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function Auth() {
    const { login, register } = useAuthStore();
    const [isLogin, setIsLogin] = useState(true);
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [serverUrl, setServerUrl] = useState("");
    const [useCustomServer, setUseCustomServer] = useState(false);
    const [isLoading, setIsLoading] = useState(false);

    // 拦截窗口关闭事件，改为隐藏
    useEffect(() => {
        const setupCloseInterceptor = async () => {
            try {
                const authWindow = getCurrentWindow();

                // 监听关闭请求，阻止默认关闭行为，改为隐藏
                const unlisten = await authWindow.onCloseRequested(
                    async (event) => {
                        event.preventDefault(); // 阻止默认的关闭行为
                        await authWindow.hide(); // 改为隐藏窗口
                    },
                );

                return unlisten;
            } catch (error) {
                return () => {};
            }
        };

        const unlistenPromise = setupCloseInterceptor();

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, []);

    const handleClose = async () => {
        const authWindow = getCurrentWindow();
        await authWindow.hide(); // 使用 hide 而不是 close
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        if (!email || !password) {
            toast.error("请填写邮箱和密码");
            return;
        }

        if (useCustomServer && !serverUrl) {
            toast.error("请填写服务器地址");
            return;
        }

        setIsLoading(true);
        try {
            // 只有选择自定义服务器时才传入 serverUrl
            const serverUrlToUse = useCustomServer ? serverUrl : undefined;

            if (isLogin) {
                await login(email, password, serverUrlToUse);
                toast.success("登录成功");

                // 通知主窗口刷新认证状态
                await emit("auth-state-changed");

                // 登录成功后关闭窗口
                setTimeout(() => {
                    handleClose();
                }, 500);
            } else {
                await register(email, password, serverUrlToUse);
                toast.success("注册成功");

                // 通知主窗口刷新认证状态
                await emit("auth-state-changed");

                setTimeout(() => {
                    handleClose();
                }, 500);
            }
        } catch (error) {
            // Tauri 返回的错误是 string 类型，不是 Error 对象
            const errorMsg = typeof error === 'string' ? error : String(error);

            console.error("[Auth.tsx] 错误详情:", errorMsg);

            toast.error(isLogin ? "登录失败" : "注册失败", {
                description: errorMsg,
            });
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="h-full flex flex-col bg-background">
            {/* 自定义标题栏 */}
            <div className="flex items-center justify-between px-4 py-2 border-b">
                <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded bg-primary" />
                    <span className="text-sm font-medium">Markdown Notes</span>
                </div>
                <button
                    onClick={handleClose}
                    className="p-1 hover:bg-muted rounded transition-colors"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>

            {/* 登录/注册表单 */}
            <div className="flex-1 p-6">
                <Tabs
                    defaultValue="login"
                    className="w-full"
                    onValueChange={(v) => setIsLogin(v === "login")}
                >
                    <TabsList className="grid w-full grid-cols-2">
                        <TabsTrigger value="login">登录</TabsTrigger>
                        <TabsTrigger value="register">注册</TabsTrigger>
                    </TabsList>

                    <TabsContent value="login" className="space-y-4 mt-6">
                        <div className="space-y-2">
                            <Label htmlFor="login-email">邮箱</Label>
                            <Input
                                id="login-email"
                                type="email"
                                placeholder="your@email.com"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                disabled={isLoading}
                            />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="login-password">密码</Label>
                            <Input
                                id="login-password"
                                type="password"
                                placeholder="••••••••"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                disabled={isLoading}
                            />
                        </div>

                        <div className="space-y-2">
                            <div className="flex items-center space-x-2">
                                <Checkbox
                                    id="login-custom-server"
                                    checked={useCustomServer}
                                    onCheckedChange={(checked) =>
                                        setUseCustomServer(checked === true)
                                    }
                                    disabled={isLoading}
                                />
                                <Label
                                    htmlFor="login-custom-server"
                                    className="text-sm cursor-pointer"
                                >
                                    使用自定义服务器
                                </Label>
                            </div>

                            {useCustomServer && (
                                <div className="space-y-2">
                                    <Label htmlFor="login-server">
                                        服务器地址
                                    </Label>
                                    <Input
                                        id="login-server"
                                        placeholder="http://localhost:3000"
                                        value={serverUrl}
                                        onChange={(e) =>
                                            setServerUrl(e.target.value)
                                        }
                                        disabled={isLoading}
                                    />
                                </div>
                            )}
                        </div>

                        <Button
                            onClick={handleSubmit}
                            className="w-full"
                            disabled={isLoading}
                        >
                            {isLoading ? "登录中..." : "登录"}
                        </Button>
                    </TabsContent>

                    <TabsContent value="register" className="space-y-4 mt-6">
                        <div className="space-y-2">
                            <Label htmlFor="register-email">邮箱</Label>
                            <Input
                                id="register-email"
                                type="email"
                                placeholder="your@email.com"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                disabled={isLoading}
                            />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="register-password">密码</Label>
                            <Input
                                id="register-password"
                                type="password"
                                placeholder="••••••••"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                disabled={isLoading}
                            />
                        </div>

                        <div className="space-y-2">
                            <div className="flex items-center space-x-2">
                                <Checkbox
                                    id="register-custom-server"
                                    checked={useCustomServer}
                                    onCheckedChange={(checked) =>
                                        setUseCustomServer(checked === true)
                                    }
                                    disabled={isLoading}
                                />
                                <Label
                                    htmlFor="register-custom-server"
                                    className="text-sm cursor-pointer"
                                >
                                    使用自定义服务器
                                </Label>
                            </div>

                            {useCustomServer && (
                                <div className="space-y-2">
                                    <Label htmlFor="register-server">
                                        服务器地址
                                    </Label>
                                    <Input
                                        id="register-server"
                                        placeholder="http://localhost:3000"
                                        value={serverUrl}
                                        onChange={(e) =>
                                            setServerUrl(e.target.value)
                                        }
                                        disabled={isLoading}
                                    />
                                </div>
                            )}
                        </div>

                        <Button
                            onClick={handleSubmit}
                            className="w-full"
                            disabled={isLoading}
                        >
                            {isLoading ? "注册中..." : "注册"}
                        </Button>
                    </TabsContent>
                </Tabs>
            </div>
        </div>
    );
}
