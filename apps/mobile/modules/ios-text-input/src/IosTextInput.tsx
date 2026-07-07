import { requireNativeViewManager } from "expo-modules-core";
import {
  ForwardRefExoticComponent,
  RefAttributes,
  useCallback,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import {
  Keyboard,
  Platform,
  StyleProp,
  StyleSheet,
  TextInput,
  TextStyle,
  type TextInputProps,
} from "react-native";

let activeInputBlur: (() => void) | null = null;

const originalKeyboardDismiss = Keyboard.dismiss;
Keyboard.dismiss = () => {
  activeInputBlur?.();
  originalKeyboardDismiss();
};

type NativeEvent<T> = { nativeEvent: T & { eventCount?: number } };
type ContentSize = { width: number; height: number };
type Selection = { start: number; end: number };

type NativeIosTextInputProps = {
  mostRecentEventCount: number;
  text?: string;
  multiline?: boolean;
  placeholder?: string;
  placeholderTextColor?: string;
  secureTextEntry?: boolean;
  editable?: boolean;
  maxLength?: number;
  showDoneButton?: boolean;

  // Appearance
  textColor?: string;
  fontSize?: number;
  textAlign?: string;
  selectionColor?: TextInputProps["selectionColor"];
  caretHidden?: boolean;

  // Keyboard / input traits
  autoCapitalize?: string;
  autoCorrect?: boolean;
  spellCheck?: boolean;
  keyboardType?: string;
  returnKeyType?: string;
  keyboardAppearance?: string;
  enablesReturnKeyAutomatically?: boolean;
  showSoftInputOnFocus?: boolean;

  // Text content / autofill
  textContentType?: string;
  passwordRules?: TextInputProps["passwordRules"];

  // Behavior
  submitBehavior?: string;
  autoFocus?: boolean;
  contextMenuHidden?: boolean;
  dismissOnTapOutside?: boolean;
  clearTextOnFocus?: boolean;
  selectTextOnFocus?: boolean;
  selection?: Selection;

  // Single-line only
  clearButtonMode?: string;

  // Multiline only
  scrollEnabled?: boolean;
  numberOfLines?: number;
  smartInsertDelete?: boolean;

  // Events
  onTextChange?: (event: NativeEvent<{ text: string }>) => void;
  onInputFocus?: (event: NativeEvent<Record<string, never>>) => void;
  onInputBlur?: (event: NativeEvent<Record<string, never>>) => void;
  onInputSubmitEditing?: (event: NativeEvent<{ text: string }>) => void;
  onInputEndEditing?: (event: NativeEvent<{ text: string }>) => void;
  onInputChange?: (event: NativeEvent<{ text: string }>) => void;
  onInputSelectionChange?: (
    event: NativeEvent<{ selection: Selection }>,
  ) => void;
  onInputKeyPress?: (event: NativeEvent<{ key: string }>) => void;
  onContentSizeChange?: (
    event: NativeEvent<{ contentSize: ContentSize }>,
  ) => void;
  onInputPress?: (event: NativeEvent<Record<string, never>>) => void;

  style?: StyleProp<TextStyle>;
};

const NativeView = requireNativeViewManager(
  "IosTextInput",
) as ForwardRefExoticComponent<
  NativeIosTextInputProps & React.RefAttributes<IosTextInputRef>
>;

const INPUT_MODE_TO_KEYBOARD_TYPE: Record<
  NonNullable<TextInputProps["inputMode"]>,
  TextInputProps["keyboardType"]
> = {
  none: "default",
  text: "default",
  decimal: "decimal-pad",
  numeric: "numeric",
  tel: "phone-pad",
  search: "web-search",
  email: "email-address",
  url: "url",
};

const ENTER_KEY_HINT_TO_RETURN_KEY: Record<
  string,
  TextInputProps["returnKeyType"]
> = {
  enter: "default",
  done: "done",
  next: "next",
  search: "search",
  send: "send",
  go: "go",
  previous: "default", // iOS has no "previous"
};

const AUTO_COMPLETE_TO_TEXT_CONTENT_TYPE: Record<string, string> = {
  username: "username",
  password: "password",
  "current-password": "password",
  "new-password": "newPassword",
  email: "emailAddress",
  "one-time-code": "oneTimeCode",
  name: "name",
  "given-name": "givenName",
  "family-name": "familyName",
  "additional-name": "middleName",
  "honorific-prefix": "namePrefix",
  "honorific-suffix": "nameSuffix",
  nickname: "nickname",
  "cc-number": "creditCardNumber",
  "postal-code": "postalCode",
  "street-address": "fullStreetAddress",
  "address-line1": "streetAddressLine1",
  "address-line2": "streetAddressLine2",
  tel: "telephoneNumber",
  url: "URL",
  country: "countryName",
  off: "none",
};

export type IosTextInputRef = {
  focus: () => void;
  blur: () => void;
  clear: () => void;
  isFocused: () => boolean;
};

export type IosTextInputProps = {
  /** Controlled text value. Omit for uncontrolled mode (use defaultValue). */
  value?: string;
  /** Initial value for uncontrolled usage. */
  defaultValue?: string;
  onChangeText?: (text: string) => void;
  multiline?: boolean;
  placeholder?: string;
  placeholderTextColor?: string;
  secureTextEntry?: boolean;
  editable?: boolean;
  /** Alias for `editable={false}`. */
  readOnly?: boolean;
  maxLength?: number;
  showDoneButton?: boolean;

  // Appearance
  textColor?: string;
  fontSize?: number;
  textAlign?: TextInputProps["textAlign"];
  selectionColor?: TextInputProps["selectionColor"];
  caretHidden?: boolean;

  // Keyboard / input traits
  autoCapitalize?: TextInputProps["autoCapitalize"];
  autoCorrect?: boolean;
  spellCheck?: boolean;
  keyboardType?: TextInputProps["keyboardType"];
  /** Higher-level alternative to keyboardType (takes precedence). */
  inputMode?: TextInputProps["inputMode"];
  returnKeyType?: TextInputProps["returnKeyType"];
  /** Higher-level alternative to returnKeyType (takes precedence). */
  enterKeyHint?: TextInputProps["enterKeyHint"];
  keyboardAppearance?: TextInputProps["keyboardAppearance"];
  enablesReturnKeyAutomatically?: boolean;
  showSoftInputOnFocus?: boolean;

  // Text content / autofill
  textContentType?: TextInputProps["textContentType"];
  /** Maps to textContentType on iOS. */
  autoComplete?: TextInputProps["autoComplete"];
  passwordRules?: TextInputProps["passwordRules"];

  // Behavior
  submitBehavior?: TextInputProps["submitBehavior"];
  autoFocus?: boolean;
  contextMenuHidden?: boolean;
  /** Dismiss keyboard when tapping outside the input. Default: true */
  dismissOnTapOutside?: boolean;
  clearTextOnFocus?: boolean;
  selectTextOnFocus?: boolean;
  selection?: Selection;

  // Single-line only
  clearButtonMode?: TextInputProps["clearButtonMode"];

  // Multiline only
  scrollEnabled?: boolean;
  numberOfLines?: number;

  // Events
  onFocus?: () => void;
  onBlur?: () => void;
  onSubmitEditing?: (text: string) => void;
  onEndEditing?: (text: string) => void;
  onChange?: (text: string) => void;
  onSelectionChange?: (selection: Selection) => void;
  onKeyPress?: (key: string) => void;
  onContentSizeChange?: (contentSize: ContentSize) => void;
  onPress?: () => void;

  style?: StyleProp<TextStyle>;
};

export const IosTextInput = ({
  autoComplete,
  defaultValue,
  editable,
  enterKeyHint,
  inputMode,
  keyboardType: keyboardTypeProp,
  multiline,
  onBlur,
  onChange,
  onChangeText,
  onContentSizeChange,
  onEndEditing,
  onFocus,
  onKeyPress,
  onPress,
  onSelectionChange,
  onSubmitEditing,
  readOnly,
  ref,
  returnKeyType: returnKeyTypeProp,
  showDoneButton,
  style,
  textContentType: textContentTypeProp,
  value,
  ...rest
}: IosTextInputProps & RefAttributes<IosTextInputRef>) => {
  // Uncontrolled state (used when `value` is omitted)
  const [internalValue, setInternalValue] = useState(defaultValue ?? "");
  const isControlled = value !== undefined;
  const textValue = isControlled ? value : internalValue;

  const [isFocusedState, setIsFocusedState] = useState(false);

  const [height, setHeight] = useState<number | undefined>(undefined);
  const nativeRef = useRef<TextInput>(null);
  const fallbackRef = useRef<TextInput>(null);
  const mostRecentEventCount = useRef(0);

  useImperativeHandle(ref, () => ({
    focus: () => {
      try {
        const promise =
          Platform.OS === "ios"
            ? (nativeRef.current?.focus() as unknown as
                | Promise<void>
                | undefined)
            : (fallbackRef.current?.focus() as any);
        promise?.catch?.(() => {});
      } catch (e) {}
    },
    blur: () => {
      try {
        const promise =
          Platform.OS === "ios"
            ? (nativeRef.current?.blur() as unknown as
                | Promise<void>
                | undefined)
            : (fallbackRef.current?.blur() as any);
        promise?.catch?.(() => {});
      } catch (e) {}
    },
    clear: () => {
      try {
        const promise =
          Platform.OS === "ios"
            ? (nativeRef.current?.clear() as unknown as
                | Promise<void>
                | undefined)
            : (fallbackRef.current?.clear() as any);
        promise?.catch?.(() => {});
      } catch (e) {}
    },
    isFocused: () => {
      return Platform.OS === "ios"
        ? isFocusedState
        : (fallbackRef.current?.isFocused() ?? false);
    },
  }));

  // Resolve higher-level props → lower-level props
  const resolvedEditable = readOnly != null ? !readOnly : editable;
  const resolvedKeyboardType =
    inputMode != null
      ? (INPUT_MODE_TO_KEYBOARD_TYPE[inputMode] ?? keyboardTypeProp)
      : keyboardTypeProp;
  const resolvedReturnKeyType =
    enterKeyHint != null
      ? (ENTER_KEY_HINT_TO_RETURN_KEY[enterKeyHint] ?? returnKeyTypeProp)
      : returnKeyTypeProp;
  const resolvedTextContentType =
    autoComplete != null && textContentTypeProp == null
      ? (AUTO_COMPLETE_TO_TEXT_CONTENT_TYPE[autoComplete] ?? undefined)
      : textContentTypeProp;

  const handleChangeText = useCallback(
    (text: string, eventCount?: number) => {
      if (eventCount !== undefined) {
        mostRecentEventCount.current = eventCount;
      }
      if (!isControlled) setInternalValue(text);
      onChangeText?.(text);
    },
    [isControlled, onChangeText],
  );

  const handleContentSizeChange = useCallback(
    (event: NativeEvent<{ contentSize: ContentSize }>) => {
      const { contentSize } = event.nativeEvent;
      setHeight(contentSize.height);
      onContentSizeChange?.(contentSize);
    },
    [onContentSizeChange],
  );

  if (Platform.OS !== "ios") {
    const {
      textColor,
      fontSize,
      clearButtonMode: _clearButtonMode,
      scrollEnabled: _scrollEnabled,
      ...textInputProps
    } = rest;
    return (
      <TextInput
        ref={fallbackRef}
        style={[
          !multiline && styles.singleLineHeight,
          textColor != null && { color: textColor },
          fontSize != null && { fontSize },
          style,
        ]}
        value={textValue}
        defaultValue={defaultValue}
        multiline={multiline}
        editable={resolvedEditable}
        keyboardType={resolvedKeyboardType}
        returnKeyType={resolvedReturnKeyType}
        textContentType={
          resolvedTextContentType as TextInputProps["textContentType"]
        }
        onChangeText={handleChangeText}
        onFocus={onFocus}
        onBlur={onBlur}
        onSubmitEditing={
          onSubmitEditing
            ? (e) => onSubmitEditing(e.nativeEvent.text)
            : undefined
        }
        onEndEditing={
          onEndEditing ? (e) => onEndEditing(e.nativeEvent.text) : undefined
        }
        onChange={onChange ? (e) => onChange(e.nativeEvent.text) : undefined}
        onSelectionChange={
          onSelectionChange
            ? (e) => onSelectionChange(e.nativeEvent.selection)
            : undefined
        }
        onKeyPress={
          onKeyPress ? (e) => onKeyPress(e.nativeEvent.key) : undefined
        }
        onContentSizeChange={
          onContentSizeChange
            ? (e) => onContentSizeChange(e.nativeEvent.contentSize)
            : undefined
        }
        onPress={onPress ? () => onPress() : undefined}
        {...textInputProps}
      />
    );
  }

  return (
    <NativeView
      ref={nativeRef}
      editable={resolvedEditable}
      keyboardType={resolvedKeyboardType}
      mostRecentEventCount={mostRecentEventCount.current}
      multiline={multiline}
      onContentSizeChange={multiline ? handleContentSizeChange : undefined}
      onInputBlur={() => {
        setIsFocusedState(false);
        activeInputBlur = null;
        onBlur?.();
      }}
      onInputChange={
        onChange
          ? (e) => {
              if (e.nativeEvent.eventCount !== undefined) {
                mostRecentEventCount.current = e.nativeEvent.eventCount;
              }
              onChange(e.nativeEvent.text);
            }
          : undefined
      }
      onInputEndEditing={
        onEndEditing ? (e) => onEndEditing(e.nativeEvent.text) : undefined
      }
      onInputFocus={() => {
        setIsFocusedState(true);
        activeInputBlur = () => {
          try {
            const promise = nativeRef.current?.blur() as unknown as
              | Promise<void>
              | undefined;
            promise?.catch?.(() => {});
          } catch (e) {}
        };
        onFocus?.();
      }}
      onInputKeyPress={
        onKeyPress ? (e) => onKeyPress(e.nativeEvent.key) : undefined
      }
      onInputPress={onPress ? () => onPress() : undefined}
      onInputSelectionChange={
        onSelectionChange
          ? (e) => onSelectionChange(e.nativeEvent.selection)
          : undefined
      }
      onInputSubmitEditing={
        onSubmitEditing ? (e) => onSubmitEditing(e.nativeEvent.text) : undefined
      }
      onTextChange={(e) =>
        handleChangeText(e.nativeEvent.text, e.nativeEvent.eventCount)
      }
      returnKeyType={resolvedReturnKeyType}
      showDoneButton={showDoneButton}
      style={[
        !multiline && styles.singleLineHeight,
        multiline && height != null && { height },
        style,
      ]}
      text={textValue}
      textContentType={resolvedTextContentType}
      {...rest}
    />
  );
};

IosTextInput.displayName = "IosTextInput";

const styles = StyleSheet.create({
  singleLineHeight: {
    height: 40,
  },
});
