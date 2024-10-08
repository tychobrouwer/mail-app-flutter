import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show TextInputFormatter;

import '../types/project_colors.dart';

class CustomFormField extends StatefulWidget {
  final void Function(String?)? onSaved;
  final String? hintText;
  final String? labelText;
  final String? Function(String?)? validator;
  final bool obscureText;
  final TextInputType keyboardType;
  final List<TextInputFormatter> inputFormatters;

  const CustomFormField({
    super.key,
    this.onSaved,
    this.hintText,
    this.labelText,
    this.validator,
    this.obscureText = false,
    this.keyboardType = TextInputType.text,
    this.inputFormatters = const [],
  });

  @override
  CustomFormFieldState createState() => CustomFormFieldState();
}

class CustomFormFieldState extends State<CustomFormField> {
  late void Function(String?)? _onSaved;
  late String? _hintText;
  late String? Function(String?)? _validator;
  late String? _labelText;
  late bool _obscureText;
  late TextInputType _keyboardType;
  late List<TextInputFormatter> _inputFormatters;

  bool _passwordVisible = false;

  @override
  void initState() {
    _onSaved = widget.onSaved;
    _hintText = widget.hintText;
    _validator = widget.validator;
    _labelText = widget.labelText;

    _obscureText = widget.obscureText;
    _keyboardType = widget.keyboardType;
    _inputFormatters = widget.inputFormatters;

    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 10),
        child: TextFormField(
          onSaved: _onSaved,
          obscureText: _passwordVisible,
          keyboardType: _keyboardType,
          inputFormatters: _inputFormatters,
          validator: _validator,
          style: TextStyle(color: ProjectColors.text(true)),
          cursorColor: ProjectColors.accent(true),
          cursorWidth: 0.6,
          decoration: InputDecoration(
            enabledBorder: UnderlineInputBorder(
              borderSide:
                  BorderSide(width: 2, color: ProjectColors.accent(false)),
            ),
            focusedBorder: UnderlineInputBorder(
              borderSide:
                  BorderSide(width: 2, color: ProjectColors.accent(true)),
            ),
            focusedErrorBorder: UnderlineInputBorder(
              borderSide:
                  BorderSide(width: 2, color: ProjectColors.accent(true)),
            ),
            errorBorder: UnderlineInputBorder(
              borderSide:
                  BorderSide(width: 2, color: ProjectColors.accent(false)),
            ),
            labelStyle: TextStyle(color: ProjectColors.text(true)),
            hintStyle: TextStyle(color: ProjectColors.text(true)),
            errorStyle: const TextStyle(fontWeight: FontWeight.w500),
            errorMaxLines: 1,
            hintText: _hintText,
            labelText: _labelText,
            errorText: '',
            suffixIcon: _obscureText
                ? IconButton(
                    splashColor: Colors.transparent,
                    icon: Icon(
                      !_passwordVisible
                          ? Icons.visibility
                          : Icons.visibility_off,
                      color: ProjectColors.accent(true),
                    ),
                    onPressed: () {
                      setState(() {
                        _passwordVisible = !_passwordVisible;
                      });
                    },
                  )
                : null,
          ),
        ),
      ),
    );
  }
}
