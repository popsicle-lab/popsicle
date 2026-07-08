import type { ReactNode } from "react";
import { useMemo } from "react";
import {
  Linking,
  Platform,
  ScrollView,
  StyleSheet,
  Text,
  View,
  type TextStyle,
  type ViewStyle,
} from "react-native";
import { marked, type Token, type Tokens } from "marked";

import { colors } from "@/theme/colors";

type ChatMarkdownVariant = "assistant" | "user";

type MarkdownTheme = {
  body: TextStyle;
  strong: TextStyle;
  em: TextStyle;
  link: TextStyle;
  muted: TextStyle;
  codeInline: TextStyle;
  codeBlock: TextStyle;
  codeWrap: ViewStyle;
  quote: ViewStyle;
  quoteText: TextStyle;
  hr: ViewStyle;
  heading: Record<number, TextStyle>;
  tableBorder: string;
  tableHeaderBg: string;
  tableHeaderText: TextStyle;
  tableCellText: TextStyle;
  mono: string;
};

function normalizeChatMarkdown(content: string): string {
  return content
    .split("\n")
    .map((line) => line.replace(/^›\s?/, ""))
    .join("\n")
    .trim();
}

function themeForVariant(variant: ChatMarkdownVariant): MarkdownTheme {
  const isUser = variant === "user";
  const text = (isUser ? "#fff" : colors.label) as string;
  const muted = (isUser ? "rgba(255,255,255,0.82)" : colors.secondaryLabel) as string;
  const codeBg = isUser ? "rgba(255,255,255,0.18)" : (colors.fill as string);
  const rule = isUser ? "rgba(255,255,255,0.28)" : (colors.separator as string);
  const link = (isUser ? "#dbeafe" : colors.systemBlue) as string;
  const mono = Platform.select({
    ios: "Menlo",
    android: "monospace",
    default: "monospace",
  })!;

  const body: TextStyle = {
    color: text,
    fontSize: 15,
    lineHeight: 21,
  };

  return {
    body,
    strong: { ...body, fontWeight: "700" },
    em: { ...body, fontStyle: "italic" },
    link: { ...body, color: link, textDecorationLine: "underline" },
    muted: { ...body, color: muted },
    codeInline: {
      fontFamily: mono,
      fontSize: 13,
      color: text,
      backgroundColor: codeBg,
      borderRadius: 4,
    },
    codeBlock: {
      fontFamily: mono,
      fontSize: 12,
      lineHeight: 17,
      color: text,
    },
    codeWrap: {
      backgroundColor: codeBg,
      borderRadius: 8,
      padding: 10,
      marginVertical: 6,
    },
    quote: {
      borderLeftWidth: 3,
      borderLeftColor: rule,
      paddingLeft: 10,
      marginVertical: 6,
    },
    quoteText: { ...body, color: muted },
    hr: {
      backgroundColor: rule,
      height: StyleSheet.hairlineWidth,
      marginVertical: 10,
    },
    heading: {
      1: { ...body, fontSize: 20, fontWeight: "700", marginBottom: 8 },
      2: { ...body, fontSize: 18, fontWeight: "700", marginBottom: 8 },
      3: { ...body, fontSize: 16, fontWeight: "600", marginBottom: 6 },
      4: { ...body, fontSize: 15, fontWeight: "600", marginBottom: 4 },
      5: { ...body, fontSize: 15, fontWeight: "600" },
      6: { ...body, fontSize: 15, fontWeight: "600", color: muted },
    },
    tableBorder: rule,
    tableHeaderBg: isUser ? "rgba(255,255,255,0.14)" : (colors.fill as string),
    tableHeaderText: { ...body, fontWeight: "600", fontSize: 14 },
    tableCellText: { ...body, fontSize: 14 },
    mono,
  };
}

function textAlignForColumn(align: ("left" | "center" | "right" | null)[] | null, index: number) {
  const value = align?.[index];
  if (value === "center") return "center" as const;
  if (value === "right") return "right" as const;
  return "left" as const;
}

function cellLayoutStyle(colCount: number, colIndex: number, scrollable: boolean): ViewStyle {
  if (scrollable) {
    return {
      width: 112,
      flexShrink: 0,
    };
  }
  if (colCount === 2) {
    return {
      flex: colIndex === 0 ? 2 : 3,
      minWidth: 0,
    };
  }
  return {
    flex: 1,
    minWidth: 0,
  };
}

function renderTableCell(
  cell: Tokens.TableCell,
  theme: MarkdownTheme,
  key: string,
  options: {
    colIndex: number;
    colCount: number;
    isHeader: boolean;
    align: ("left" | "center" | "right" | null)[] | null;
    isLastRow: boolean;
    scrollable: boolean;
  }
) {
  const { colIndex, colCount, isHeader, align, isLastRow, scrollable } = options;
  const textAlign = textAlignForColumn(align, colIndex);
  const content = renderInlineChildren(cell.tokens, theme) ?? cell.text.trim();

  return (
    <View
      key={key}
      style={[
        cellLayoutStyle(colCount, colIndex, scrollable),
        {
          paddingHorizontal: 8,
          paddingVertical: 6,
          borderRightWidth: colIndex < colCount - 1 ? StyleSheet.hairlineWidth : 0,
          borderRightColor: theme.tableBorder,
          borderBottomWidth: isLastRow ? 0 : StyleSheet.hairlineWidth,
          borderBottomColor: theme.tableBorder,
          backgroundColor: isHeader ? theme.tableHeaderBg : undefined,
        },
      ]}
    >
      <Text
        style={[
          isHeader ? theme.tableHeaderText : theme.tableCellText,
          { textAlign },
        ]}
        selectable
      >
        {content || " "}
      </Text>
    </View>
  );
}

function renderTable(token: Tokens.Table, theme: MarkdownTheme, key: string): ReactNode {
  const colCount = token.header.length;
  const scrollable = colCount > 3;

  const tableBody = (
    <View
      style={{
        borderWidth: StyleSheet.hairlineWidth,
        borderColor: theme.tableBorder,
        borderRadius: 8,
        overflow: "hidden",
        width: scrollable ? undefined : "100%",
      }}
    >
      <View style={{ flexDirection: "row", alignItems: "flex-start" }}>
        {token.header.map((cell, colIndex) =>
          renderTableCell(cell, theme, `${key}-h-${colIndex}`, {
            colIndex,
            colCount,
            isHeader: true,
            align: token.align,
            isLastRow: false,
            scrollable,
          })
        )}
      </View>
      {token.rows.map((row, rowIndex) => (
        <View
          key={`${key}-r-${rowIndex}`}
          style={{ flexDirection: "row", alignItems: "flex-start" }}
        >
          {row.map((cell, colIndex) =>
            renderTableCell(cell, theme, `${key}-r-${rowIndex}-c-${colIndex}`, {
              colIndex,
              colCount,
              isHeader: false,
              align: token.align,
              isLastRow: rowIndex === token.rows.length - 1,
              scrollable,
            })
          )}
        </View>
      ))}
    </View>
  );

  return (
    <View key={key} style={{ marginVertical: 6, alignSelf: "stretch" }}>
      {scrollable ? (
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator
          contentContainerStyle={{ flexGrow: 0 }}
        >
          {tableBody}
        </ScrollView>
      ) : (
        tableBody
      )}
    </View>
  );
}

function renderTextToken(token: Tokens.Text, theme: MarkdownTheme, key: string): ReactNode {
  return (
    <Text key={key} style={theme.body} selectable>
      {token.tokens?.length
        ? renderInlineChildren(token.tokens, theme)
        : token.text}
    </Text>
  );
}

function renderInline(tokens: Token[] | undefined, theme: MarkdownTheme, key: string): ReactNode {
  if (!tokens?.length) return null;
  return (
    <Text key={key} style={theme.body} selectable>
      {tokens.map((token, index) => renderInlineToken(token, theme, `${key}-${index}`))}
    </Text>
  );
}

function renderInlineToken(token: Token, theme: MarkdownTheme, key: string): ReactNode {
  switch (token.type) {
    case "text":
      return token.text;
    case "strong":
      return (
        <Text key={key} style={theme.strong}>
          {renderInlineChildren(token.tokens, theme)}
        </Text>
      );
    case "em":
      return (
        <Text key={key} style={theme.em}>
          {renderInlineChildren(token.tokens, theme)}
        </Text>
      );
    case "del":
      return (
        <Text key={key} style={{ textDecorationLine: "line-through" }}>
          {renderInlineChildren(token.tokens, theme)}
        </Text>
      );
    case "codespan":
      return (
        <Text key={key} style={theme.codeInline}>
          {token.text}
        </Text>
      );
    case "link":
      return (
        <Text
          key={key}
          style={theme.link}
          onPress={() => Linking.openURL(token.href).catch(() => {})}
        >
          {renderInlineChildren(token.tokens, theme) ?? token.text}
        </Text>
      );
    case "image":
      return (
        <Text key={key} style={theme.link}>
          {token.text || token.href}
        </Text>
      );
    case "br":
      return "\n";
    default: {
      const nested = "tokens" in token ? (token as Tokens.Generic).tokens : undefined;
      if (nested?.length) {
        return renderInlineChildren(nested, theme);
      }
      if ("text" in token && typeof token.text === "string") {
        return token.text;
      }
      return "raw" in token && typeof token.raw === "string" ? token.raw : null;
    }
  }
}

function renderInlineChildren(tokens: Token[] | undefined, theme: MarkdownTheme): ReactNode {
  if (!tokens?.length) return null;
  return tokens.map((token, index) => renderInlineToken(token, theme, `c-${index}`));
}

function renderBlocks(tokens: Token[], theme: MarkdownTheme): ReactNode[] {
  return tokens.map((token, index) => renderBlockToken(token, theme, `b-${index}`));
}

function renderBlockToken(token: Token, theme: MarkdownTheme, key: string): ReactNode {
  switch (token.type) {
    case "paragraph":
      return (
        <View key={key} style={{ marginBottom: 8 }}>
          {renderInline(token.tokens, theme, `${key}-p`)}
        </View>
      );
    case "heading": {
      const depth = (token as Tokens.Heading).depth;
      return (
        <Text
          key={key}
          style={theme.heading[depth] ?? theme.heading[6]}
          selectable
        >
          {renderInlineChildren(token.tokens, theme)}
        </Text>
      );
    }
    case "blockquote":
      return (
        <View key={key} style={theme.quote}>
          {(token.tokens ?? []).map((child, index) =>
            child.type === "paragraph" ? (
              <Text key={`${key}-q-${index}`} style={theme.quoteText} selectable>
                {renderInlineChildren(child.tokens, theme)}
              </Text>
            ) : (
              renderBlockToken(child, theme, `${key}-q-${index}`)
            )
          )}
        </View>
      );
    case "code":
      return (
        <View key={key} style={theme.codeWrap}>
          <Text style={theme.codeBlock} selectable>
            {token.text.replace(/\n$/, "")}
          </Text>
        </View>
      );
    case "hr":
      return <View key={key} style={theme.hr} />;
    case "space":
      return <View key={key} style={{ height: 8 }} />;
    case "table":
      return renderTable(token as Tokens.Table, theme, key);
    case "list": {
      const list = token as Tokens.List;
      return (
        <View key={key} style={{ marginBottom: 8, gap: 4 }}>
          {list.items.map((item: Tokens.ListItem, index: number) => (
            <View key={`${key}-li-${index}`} style={{ flexDirection: "row", gap: 8 }}>
              <Text style={theme.body} selectable>
                {list.ordered ? `${index + 1}.` : "•"}
              </Text>
              <View style={{ flex: 1, gap: 4 }}>
                {item.tokens.map((child: Token, childIndex: number) =>
                  child.type === "text" ? (
                    renderTextToken(
                      child as Tokens.Text,
                      theme,
                      `${key}-lit-${childIndex}`
                    )
                  ) : (
                    renderBlockToken(child, theme, `${key}-lic-${childIndex}`)
                  )
                )}
              </View>
            </View>
          ))}
        </View>
      );
    }
    case "text":
      return renderTextToken(token as Tokens.Text, theme, key);
    default:
      return "raw" in token && typeof token.raw === "string" ? (
        <Text key={key} style={theme.body}>
          {token.raw}
        </Text>
      ) : null;
  }
}

export function ChatMarkdown({
  content,
  variant = "assistant",
}: {
  content: string;
  variant?: ChatMarkdownVariant;
}) {
  const theme = useMemo(() => themeForVariant(variant), [variant]);
  const blocks = useMemo(() => {
    const markdown = normalizeChatMarkdown(content);
    if (!markdown) return [];
    return marked.lexer(markdown, { gfm: true, breaks: true });
  }, [content]);

  if (!blocks.length) return null;

  return <View style={{ alignSelf: "stretch", width: "100%" }}>{renderBlocks(blocks, theme)}</View>;
}
