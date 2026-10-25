# aviutl2-script-skill

Claude Code 技能，为 [AviUtl2](https://github.com/aviutl/aviutl2) / ExEdit2 提供 Lua 脚本编写指导。

支持 **5 种**脚本类型：

| 扩展名  | 用途                       |
|---------|---------------------------|
| `.anm2` | 动画效果（フィルタ効果）    |
| `.obj2` | 自定义对象（カスタムオブジェクト）|
| `.cam2` | 摄像机控制（カメラ効果）    |
| `.scn2` | 场景切换（シーンチェンジ）   |
| `.tra2` | 轨道移动（トラックバー移動） |

## 目录结构

```
.
├── SKILL.md                  # 技能主文件（变量、指令、模式速查）
├── references/
│   ├── api.md                # 完整 API 参考（所有函数签名与参数）
│   └── examples.md           # 可运行示例集
├── assets/
│   ├── template.anm2         # 动画效果模板
│   ├── template.obj2         # 自定义对象模板
│   ├── template.cam2         # 摄像机效果模板
│   ├── template.scn2         # 场景切换模板
│   └── template.tra2         # 轨道移动模板
└── LICENSE
```

## 使用

将本技能安装到 Claude Code 中后，触发词会自动激活：

- **中文**：AviUtl2、ExEdit2、动画效果、脚本效果
- **日语**：フィルタ効果
- **扩展名**：anm2、obj2、cam2、scn2、tra2

## 模板快速开始

以动画效果为例，复制 `assets/template.anm2` 到 AviUtl2 的 `Script\` 目录：

```lua
-- 右移 + 旋转
obj.ox = obj.ox + speed * obj.time
obj.rz = obj.rz + obj.time * 360
```

更多用法见 `references/examples.md`。

## License

MIT
