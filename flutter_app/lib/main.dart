import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';

void main(List<String> args) {
  runApp(SaveConverterApp(arguments: args));
}

class SaveConverterApp extends StatelessWidget {
  const SaveConverterApp({super.key, required this.arguments});

  final List<String> arguments;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'REE Save Converter',
      theme: ThemeData(colorSchemeSeed: Colors.deepPurple, useMaterial3: true),
      home: ConverterScreen(arguments: arguments),
    );
  }
}

class ConverterScreen extends StatefulWidget {
  const ConverterScreen({super.key, required this.arguments});

  final List<String> arguments;

  @override
  State<ConverterScreen> createState() => _ConverterScreenState();
}

class _ConverterScreenState extends State<ConverterScreen> {
  late final _LaunchConfig _config;
  Process? _process;
  bool _starting = false;
  String _status = 'Готово к конвертации';

  bool get _isBusy => _starting || _process != null;

  @override
  void initState() {
    super.initState();
    _config = _LaunchConfig.fromArgs(widget.arguments);
  }

  Future<void> _convertSaves() async {
    if (_isBusy) {
      return;
    }

    setState(() {
      _starting = true;
      _status = 'Запуск конвертации…';
    });

    await Future<void>.delayed(Duration.zero);
    if (!mounted || !_starting) {
      return;
    }

    try {
      final process = await Process.start(
        _config.backendExecutable,
        _config.backendArguments,
        mode: ProcessStartMode.normal,
      );

      if (!mounted || !_starting) {
        process.kill();
        return;
      }

      setState(() {
        _starting = false;
        _process = process;
        _status = 'Конвертация выполняется…';
      });

      final exitCode = await process.exitCode;
      if (!mounted) {
        return;
      }

      setState(() {
        _process = null;
        _status = exitCode == 0
            ? 'Конвертация завершена'
            : 'Конвертация завершилась с ошибкой: $exitCode';
      });
    } catch (error) {
      if (!mounted) {
        return;
      }

      setState(() {
        _starting = false;
        _process = null;
        _status = 'Не удалось запустить конвертацию: $error';
      });
    }
  }

  void _cancel() {
    if (_process != null) {
      setState(() {
        _status = 'Конвертация уже запущена; дождитесь завершения процесса';
      });
      return;
    }

    if (_starting) {
      setState(() {
        _starting = false;
        _status = 'Запуск конвертации отменён';
      });
      return;
    }

    exit(0);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 360),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              stretch: CrossAxisAlignment.stretch,
              children: [
                FilledButton(
                  onPressed: _isBusy ? null : _convertSaves,
                  child: const Text('Конвертировать сейвы'),
                ),
                const SizedBox(height: 12),
                OutlinedButton(
                  onPressed: _cancel,
                  child: const Text('Отмена'),
                ),
                const SizedBox(height: 24),
                Text(
                  _status,
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _LaunchConfig {
  const _LaunchConfig({
    required this.backendExecutable,
    required this.backendArguments,
  });

  factory _LaunchConfig.fromArgs(List<String> args) {
    final backendArguments = <String>[];
    var backendExecutable = 'ree-save-editor';

    for (var index = 0; index < args.length; index += 1) {
      final argument = args[index];
      if (argument == '--backend' && index + 1 < args.length) {
        backendExecutable = args[++index];
      } else if (argument.startsWith('--backend=')) {
        backendExecutable = argument.substring('--backend='.length);
      } else {
        backendArguments.add(argument);
      }
    }

    if (!backendArguments.contains('--save')) {
      backendArguments.add('--save');
    }

    return _LaunchConfig(
      backendExecutable: backendExecutable,
      backendArguments: List.unmodifiable(backendArguments),
    );
  }

  final String backendExecutable;
  final List<String> backendArguments;
}
