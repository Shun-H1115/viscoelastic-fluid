# 🎯 狙撃できる粘弾性水風船シミュレーション（Rust + macroquad）

このプロジェクトは、Rust と[`macroquad`](https://github.com/not-fl3/macroquad)を使った、  
「静止した真球状の水風船が、クリックで撃たれて弾ける」インタラクティブな 2D シミュレーションです。

---

## 数学的背景

このシミュレーションは、次の 3 つの力に基づく**粘弾性粒子系**として構成されています：

---

### ① 粒子の球状配置（極座標）

水風船は 1000 個の粒子から構成され、**同心円状（真球）**に分布します。

極座標による配置式：

\[
\vec{p}\_i = \vec{c} + r_k
\begin{bmatrix}
\cos(\theta_i) \\
\sin(\theta_i)
\end{bmatrix},\quad
\theta_i = \frac{2\pi i}{N_k}
\]

- \( \vec{c} \)：中心座標
- \( r_k \)：第 \(k\) 層の半径
- \( N_k \)：その層の粒子数

---

### ② 粘弾性ばね力（フックの法則＋ダンピング）

近接粒子同士は仮想ばねで接続され、以下の力を持ちます：

**フックの法則（弾性）：**

\[
\vec{F}\_{\text{spring}} = -k (x - L_0) \cdot \hat{n}
\]

**ダンピング（粘性）：**

\[
\vec{F}_{\text{damping}} = -c (\vec{v}_{ij} \cdot \hat{n}) \cdot \hat{n}
\]

- \( k \)：ばね定数
- \( c \)：粘性係数
- \( x \)：現在の粒子間距離
- \( L_0 \)：自然長（初期ばね長）
- \( \hat{n} \)：粒子間の単位ベクトル
- \( \vec{v}\_{ij} \)：相対速度

---

### ③ 重力と時間発展

すべての粒子に重力がかかり、**半陰的オイラー法**で更新されます：

\[
\vec{F}\_{\text{gravity}} = m \cdot \vec{g}
\]

\[
\vec{v}_{t+1} = \vec{v}\_t + \frac{\vec{F}}{m} \cdot \Delta t,\quad
\vec{x}_{t+1} = \vec{x}_t + \vec{v}_{t+1} \cdot \Delta t
\]

---

## 実行方法（Ubuntu）

### 依存ライブラリのインストール

```bash
sudo apt update
sudo apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```
