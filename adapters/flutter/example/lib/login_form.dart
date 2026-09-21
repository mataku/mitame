import 'package:flutter/material.dart';

class LoginForm extends StatelessWidget {
  const LoginForm({super.key, this.title = 'Sign in'});

  final String title;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(title, style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 16),
          const TextField(decoration: InputDecoration(labelText: 'Email')),
          const SizedBox(height: 8),
          const TextField(
            obscureText: true,
            decoration: InputDecoration(labelText: 'Password'),
          ),
          const SizedBox(height: 16),
          FilledButton(onPressed: () {}, child: const Text('Continue')),
        ],
      ),
    );
  }
}
