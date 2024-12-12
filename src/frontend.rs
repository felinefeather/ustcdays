// frontend.rs

pub trait Frontend {
    /// 显示一段文本
    fn display_text(&self, text: &str);

    /// 显示选项并获取玩家的选择
    /// 返回玩家选择的选项索引
    fn display_options(&self, options: &[String]) -> usize;

    /// 显示玩家属性的过高或过低描述
    fn display_player_status(&self, descriptions: &[String]);

    /// 显示错误信息
    fn display_error(&self, message: &str);
}
