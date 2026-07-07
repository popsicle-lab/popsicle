import Constants from "expo-constants";
import {
  Host,
  TextField,
  useNativeState,
  type TextFieldRef,
} from "@expo/ui/swift-ui";
import { disabled, lineLimit } from "@expo/ui/swift-ui/modifiers";
import { forwardRef, memo, useImperativeHandle, useRef } from "react";
import { View } from "react-native";

import { IosTextInput, type IosTextInputRef } from "../../modules/ios-text-input";
import { colors } from "@/theme/colors";
import { spacing, typography } from "@/theme/tokens";

const INPUT_HEIGHT = 96;
const isExpoGo = Constants.appOwnership === "expo";

export type IntakeMessageInputRef = {
  getText: () => string;
  clear: () => void;
  focus: () => void;
};

type IntakeMessageInputProps = {
  placeholder?: string;
  editable?: boolean;
};

const ExpoGoIntakeInput = memo(
  forwardRef<IntakeMessageInputRef, IntakeMessageInputProps>(
    function ExpoGoIntakeInput({ placeholder, editable = true }, ref) {
      const text = useNativeState("");
      const fieldRef = useRef<TextFieldRef>(null);

      useImperativeHandle(
        ref,
        () => ({
          getText: () => text.value,
          clear: () => {
            text.value = "";
            void fieldRef.current?.clear();
          },
          focus: () => {
            void fieldRef.current?.focus();
          },
        }),
        [text]
      );

      return (
        <View style={{ width: "100%", minHeight: INPUT_HEIGHT }}>
          <Host style={{ width: "100%", height: INPUT_HEIGHT }}>
            <TextField
              ref={fieldRef}
              text={text}
              placeholder={placeholder}
              axis="vertical"
              modifiers={[
                lineLimit(4, { reservesSpace: true }),
                ...(editable ? [] : [disabled(true)]),
              ]}
            />
          </Host>
        </View>
      );
    }
  )
);

const NativeIntakeInput = memo(
  forwardRef<IntakeMessageInputRef, IntakeMessageInputProps>(
    function NativeIntakeInput({ placeholder, editable = true }, ref) {
      const inputRef = useRef<IosTextInputRef>(null);
      const textRef = useRef("");

      useImperativeHandle(ref, () => ({
        getText: () => textRef.current,
        clear: () => {
          textRef.current = "";
          inputRef.current?.clear();
        },
        focus: () => {
          inputRef.current?.focus();
        },
      }));

      return (
        <IosTextInput
          ref={inputRef}
          placeholder={placeholder}
          defaultValue=""
          onChangeText={(value) => {
            textRef.current = value;
          }}
          multiline
          editable={editable}
          scrollEnabled
          numberOfLines={4}
          fontSize={typography.body.fontSize}
          textColor={colors.label as string}
          placeholderTextColor={colors.tertiaryLabel as string}
          style={{
            minHeight: INPUT_HEIGHT,
            maxHeight: 160,
            paddingVertical: spacing.sm,
          }}
        />
      );
    }
  )
);

export const IntakeMessageInput = isExpoGo ? ExpoGoIntakeInput : NativeIntakeInput;
