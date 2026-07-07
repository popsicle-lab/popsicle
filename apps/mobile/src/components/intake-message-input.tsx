import { forwardRef, memo, useImperativeHandle, useRef } from "react";
import { TextInput, type TextInput as RNTextInput } from "react-native";

import { colors } from "@/theme/colors";
import { spacing, typography } from "@/theme/tokens";

export type IntakeMessageInputRef = {
  getText: () => string;
  clear: () => void;
  focus: () => void;
};

type IntakeMessageInputProps = {
  placeholder?: string;
  editable?: boolean;
};

export const IntakeMessageInput = memo(
  forwardRef<IntakeMessageInputRef, IntakeMessageInputProps>(
    function IntakeMessageInput({ placeholder, editable = true }, ref) {
      const inputRef = useRef<RNTextInput>(null);
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
        <TextInput
          ref={inputRef}
          placeholder={placeholder}
          defaultValue=""
          onChangeText={(value) => {
            textRef.current = value;
          }}
          multiline
          editable={editable}
          textAlignVertical="top"
          blurOnSubmit={false}
          placeholderTextColor={colors.tertiaryLabel as string}
          style={{
            ...typography.body,
            color: colors.label as string,
            paddingVertical: spacing.sm,
            minHeight: 96,
            maxHeight: 160,
          }}
        />
      );
    }
  )
);
