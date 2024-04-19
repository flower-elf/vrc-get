import {twMerge} from "tailwind-merge";
import React, {forwardRef} from "react";

export type CardProps = {
	children?: React.ReactNode,
	className?: string,
};

export default forwardRef<HTMLDivElement, CardProps>(function Card(props: CardProps, ref) {
	const className = twMerge(
		"flex flex-col " +
		"bg-clip-border rounded-xl " +
		"bg-card-bg text-text-main",
		props.className,
	)
	return (
		<div ref={ref} className={className}>
			{props.children}
		</div>
	);
});
