/**
 * classNames 合并 React className 片段。
 *
 * 该工具只处理当前项目需要的简单场景：过滤空值后用空格拼接，避免在没有额外
 * 依赖的情况下反复手写数组过滤逻辑。
 */
export function classNames(...names: Array<string | false | null | undefined>) {
  return names.filter(Boolean).join(" ");
}
